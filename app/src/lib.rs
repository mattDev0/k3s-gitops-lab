use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let http_requests_total = IntCounterVec::new(
            Opts::new(
                "http_requests_total",
                "Total number of HTTP requests processed",
            ),
            &["route", "method", "status"],
        )?;

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["route", "method", "status"],
        )?;

        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub fault_rate: f64,
    pub metrics: Metrics,
}

pub fn parse_fault_error_rate(raw: Option<&str>) -> Result<f64, String> {
    match raw {
        None => Ok(0.0),
        Some(s) if s.trim().is_empty() => Ok(0.0),
        Some(s) => {
            let val: f64 = s
                .trim()
                .parse::<f64>()
                .map_err(|e| format!("Invalid FAULT_ERROR_RATE '{s}': {e}"))?;
            if !(0.0..=1.0).contains(&val) || val.is_nan() {
                return Err(format!(
                    "FAULT_ERROR_RATE must be a float between 0.0 and 1.0, got: {val}"
                ));
            }
            Ok(val)
        }
    }
}

pub async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    let rate = state.fault_rate;
    let should_fail = if rate <= 0.0 {
        false
    } else if rate >= 1.0 {
        true
    } else {
        fastrand::f64() < rate
    };

    if should_fail {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "fault injection",
                "fault_error_rate": rate
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "service": "demo",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

pub async fn healthz_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

pub async fn readyz_handler() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = state.metrics.registry.gather();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        eprintln!("Failed to encode metrics: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Internal server error".as_bytes().to_vec(),
        );
    }

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        buffer,
    )
}

pub async fn request_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let route = match path.as_str() {
        "/" => "/".to_string(),
        "/healthz" => "/healthz".to_string(),
        "/readyz" => "/readyz".to_string(),
        "/metrics" => "/metrics".to_string(),
        _ => "unknown".to_string(),
    };

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status_code = response.status();
    let status_str = status_code.as_u16().to_string();

    if path != "/metrics" {
        state
            .metrics
            .http_requests_total
            .with_label_values(&[&route, &method, &status_str])
            .inc();

        state
            .metrics
            .http_request_duration_seconds
            .with_label_values(&[&route, &method, &status_str])
            .observe(duration.as_secs_f64());
    }

    let log_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "method": method,
        "path": path,
        "route": route,
        "status": status_code.as_u16(),
        "duration_seconds": duration.as_secs_f64(),
    });
    println!("{}", log_entry);

    response
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/healthz", get(healthz_handler))
        .route("/readyz", get(readyz_handler))
        .route("/metrics", get(metrics_handler))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            request_middleware,
        ))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;

    #[test]
    fn test_parse_fault_error_rate() {
        assert_eq!(parse_fault_error_rate(None).unwrap(), 0.0);
        assert_eq!(parse_fault_error_rate(Some("")).unwrap(), 0.0);
        assert_eq!(parse_fault_error_rate(Some("  0.0  ")).unwrap(), 0.0);
        assert_eq!(parse_fault_error_rate(Some("0.5")).unwrap(), 0.5);
        assert_eq!(parse_fault_error_rate(Some("1.0")).unwrap(), 1.0);

        assert!(parse_fault_error_rate(Some("abc")).is_err());
        assert!(parse_fault_error_rate(Some("-0.1")).is_err());
        assert!(parse_fault_error_rate(Some("1.01")).is_err());
        assert!(parse_fault_error_rate(Some("nan")).is_err());
        assert!(parse_fault_error_rate(Some("inf")).is_err());
    }

    #[tokio::test]
    async fn test_fault_rate_zero_never_fails() {
        let metrics = Metrics::new().unwrap();
        let state = AppState {
            fault_rate: 0.0,
            metrics,
        };
        let app = create_app(state);

        for _ in 0..25 {
            let req = Request::builder()
                .uri("/")
                .method("GET")
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);

            let body_bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
            assert_eq!(json["service"], "demo");
            assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
        }
    }

    #[tokio::test]
    async fn test_fault_rate_one_always_fails() {
        let metrics = Metrics::new().unwrap();
        let state = AppState {
            fault_rate: 1.0,
            metrics,
        };
        let app = create_app(state);

        for _ in 0..25 {
            let req = Request::builder()
                .uri("/")
                .method("GET")
                .body(Body::empty())
                .unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

            let body_bytes = axum::body::to_bytes(res.into_body(), 1024 * 16)
                .await
                .unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
            assert_eq!(json["error"], "fault injection");
        }
    }

    #[tokio::test]
    async fn test_health_and_readiness_endpoints_never_fail() {
        let metrics = Metrics::new().unwrap();
        let state = AppState {
            fault_rate: 1.0, // Fault injection is 100%, but health endpoints must stay 200
            metrics,
        };
        let app = create_app(state);

        let req_health = Request::builder()
            .uri("/healthz")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_health = app.clone().oneshot(req_health).await.unwrap();
        assert_eq!(res_health.status(), StatusCode::OK);

        let req_ready = Request::builder()
            .uri("/readyz")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_ready = app.clone().oneshot(req_ready).await.unwrap();
        assert_eq!(res_ready.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_recording_and_labels() {
        let metrics = Metrics::new().unwrap();
        let state = AppState {
            fault_rate: 0.0,
            metrics: metrics.clone(),
        };
        let app = create_app(state);

        // Send 1 request to /
        let req_root = Request::builder()
            .uri("/")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_root = app.clone().oneshot(req_root).await.unwrap();
        assert_eq!(res_root.status(), StatusCode::OK);

        // Send 1 request to /healthz
        let req_health = Request::builder()
            .uri("/healthz")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_health = app.clone().oneshot(req_health).await.unwrap();
        assert_eq!(res_health.status(), StatusCode::OK);

        // Send 1 request to /readyz
        let req_ready = Request::builder()
            .uri("/readyz")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_ready = app.clone().oneshot(req_ready).await.unwrap();
        assert_eq!(res_ready.status(), StatusCode::OK);

        // Send 1 request to unknown path
        let req_404 = Request::builder()
            .uri("/not-found")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_404 = app.clone().oneshot(req_404).await.unwrap();
        assert_eq!(res_404.status(), StatusCode::NOT_FOUND);

        // Now request /metrics
        let req_metrics = Request::builder()
            .uri("/metrics")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        let res_metrics = app.clone().oneshot(req_metrics).await.unwrap();
        assert_eq!(res_metrics.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(res_metrics.into_body(), 1024 * 64)
            .await
            .unwrap();
        let metrics_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Verify http_requests_total lines
        assert!(metrics_text
            .contains("http_requests_total{method=\"GET\",route=\"/\",status=\"200\"} 1"));
        assert!(metrics_text
            .contains("http_requests_total{method=\"GET\",route=\"/healthz\",status=\"200\"} 1"));
        assert!(metrics_text
            .contains("http_requests_total{method=\"GET\",route=\"/readyz\",status=\"200\"} 1"));
        assert!(metrics_text
            .contains("http_requests_total{method=\"GET\",route=\"unknown\",status=\"404\"} 1"));

        // Verify /metrics itself is not recorded in http_requests_total
        assert!(!metrics_text.contains("route=\"/metrics\""));

        // Verify http_request_duration_seconds histogram is recorded
        assert!(metrics_text.contains("http_request_duration_seconds_bucket"));
        assert!(metrics_text.contains("http_request_duration_seconds_count"));
    }
}
