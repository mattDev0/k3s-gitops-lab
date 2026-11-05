# k3s-gitops-lab

A single-node Kubernetes lab running on an 8 GB Linode VM (`dev-workspace`), built to learn and demonstrate GitOps workflows, automated release verification, Prometheus SLO burn-rate alerting, and a git-driven rollback. The service is a small Rust application deployed exclusively through Argo CD with automated drift correction, without public exposure or production traffic claims.

**Status:** Stage 3 of 5: monitoring, SLO and alerts

## Documentation

- [Architecture & Design Decisions](docs/architecture.md)
- [Service Level Objective & Burn-Rate Alerting](docs/slo.md)
- [Pinned Versions](docs/versions.md)
- [Incident Report: Bad Release Rollback](docs/incidents/2026-11-05-bad-release-rollback.md)

## Local Development

### Prerequisites

- Rust toolchain (1.99+)
- Docker (optional, for container packaging)

### Running Locally

Run the Axum demo service with Cargo:

```bash
cd app
cargo run
```

By default, the service listens on `0.0.0.0:8080`.

Endpoints:
- `GET /` — Returns JSON metadata (`{"service":"demo","version":"1.0.0"}`)
- `GET /healthz` — Liveness probe (returns 200 OK)
- `GET /readyz` — Readiness probe (returns 200 OK)
- `GET /metrics` — Prometheus metrics format (`http_requests_total`, `http_request_duration_seconds`)

Fault injection can be configured via environment variables:

```bash
# Injects a 30% failure rate (HTTP 500) on GET /
FAULT_ERROR_RATE=0.3 cargo run
```

### Building the Container

Build the container image using the multi-stage Dockerfile:

```bash
docker build -t demo:local app/
```

Run the container:

```bash
docker run --rm -p 8080:8080 demo:local
```

