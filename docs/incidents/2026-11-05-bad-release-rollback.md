# Incident Report: Bad Release Rollback (Fault Injection Exercise)

**Severity**: Low — planned exercise in a lab, synthetic traffic only.

- **Incident Date**: 2026-11-05
- **Service**: `demo` (Rust Axum HTTP API)
- **Time to Detect (TTD)**: 29m 30s (Page alert firing; Ticket alert firing at 11m 30s)
- **Time to Recover (TTR)**: 7m 00s (Page alert resolution following revert PR merge; 3m 41s after pod rollout)
- **Total Failure Duration**: 48m 19s (08:52:51 UTC to 09:41:10 UTC)

---

## 1. Summary

This incident was a planned fault-injection exercise in `k3s-gitops-lab` designed to validate automated GitOps deployments, multi-window multi-burn-rate Service Level Objective (SLO) alerting, Alertmanager notifications, and a Git-driven rollback using `git revert`.

At 08:50:34 UTC, a simulated faulty release (PR #7) was merged into `main`, setting `FAULT_ERROR_RATE: "0.3"` on the demo deployment. Argo CD synchronized the change at 08:51:00 UTC. The application began returning HTTP 500 errors to synthetic traffic at 08:52:51 UTC, causing the 5-minute error ratio to climb to ~34.3%. The ticket-level burn-rate alert fired at 09:04:21 UTC, and the page-level burn-rate alert fired at 09:22:21 UTC.

The regression was resolved by merging PR #8 at 09:37:52 UTC, which cleanly executed `git revert` on the bad release commit. Argo CD automatically detected and deployed the revert at 09:41:02 UTC. New healthy pods began serving requests at 09:41:10 UTC. The page alert resolved cleanly at 09:44:51 UTC as the 5-minute error ratio dropped below the 14.4% threshold, and the 5-minute error ratio returned to 0.00000 at 09:46:10 UTC.

---

## 2. Impact

- **Customer Impact**: None. All traffic was synthetically generated inside the cluster by the `loadgen` deployment (`curlimages/curl` sending ~2.8 to 5 rps). No user-facing systems exist on this cluster.
- **Traffic Volume & Errors**:
  - Total requests served during the 2-hour incident window: **15,672**
  - Total HTTP 5xx errors recorded: **2,518** (~16.07% error rate across the 2-hour window; ~34.3% error rate during the active 48-minute failure window)
- **Error Budget Consumption**:
  - Under a theoretical 30-day baseline (~12,960,000 requests per month at 5 rps, allowing 129,600 errors under a 99.0% SLO), the incident consumed **1.94%** of the monthly error budget ($2,518 / 129,600 \approx 0.01943$).
  - The Grafana SLO Dashboard remaining budget gauge read **0% budget left**. This overstates the damage because Prometheus only retained ~2.5 hours of total cluster traffic history since bootstrap, rather than a rolling 30-day baseline.

---

## 3. Timeline (UTC)

| Timestamp | Event |
|:---|:---|
| `08:50:34Z` | PR #7 (`release/demo-1.1.0-bad`, commit `7916067`) merged into `main`, setting `FAULT_ERROR_RATE: "0.3"`. |
| `08:50:59Z` | Argo CD begins automated reconciliation of Application `demo` at revision `7916067`. |
| `08:51:00Z` | Argo CD sync succeeds; Kubernetes begins rolling replacement of demo pods. |
| `08:51:10Z` | Faulty demo pod starts up with `fault_error_rate: 0.3`. |
| `08:52:00Z` | Prometheus request rate panel briefly registers 0 for ~30 seconds during pod replacement and initial scrape interval. |
| `08:52:51Z` | First HTTP 500 error logged by demo container. |
| `08:53:09Z` | Prometheus rule evaluates `job:http_requests_error_ratio:rate5m` at `0.0342` (3.42%). |
| `08:54:32Z` | `rate5m` crosses `0.144` (14.4% error ratio, reaching `0.176`). |
| `08:57:02Z` | `rate5m` stabilizes at steady-state failure rate of `0.343` (34.3%). |
| `09:02:21Z` | `DemoServiceErrorBudgetBurnRateSlow` (ticket) alert enters `pending` state (`rate6h` and `rate30m` both exceed 0.06). |
| `09:04:21Z` | `DemoServiceErrorBudgetBurnRateSlow` (ticket) enters `firing` state (after `for: 2m`). |
| `09:20:21Z` | `DemoServiceErrorBudgetBurnRateHigh` (page) alert enters `pending` state (`rate1h` reaches `0.1477` > `0.144` while `rate5m` is `0.343`). |
| `09:22:21Z` | `DemoServiceErrorBudgetBurnRateHigh` (page) enters `firing` state (after `for: 2m`). |
| `09:30:00Z` | Screenshots captured of firing alerts in Prometheus, Alertmanager, and Grafana. |
| `09:35:00Z` | Branch `fix/revert-1.1.0` created; `git revert -m 1 7916067` executed cleanly; PR #8 opened. |
| `09:37:52Z` | PR #8 merged into `main` by Matt (merge commit `38c6642`). |
| `09:41:01Z` | Argo CD begins automated sync of Application `demo` at revision `38c6642`. |
| `09:41:02Z` | Argo CD sync finishes successfully; deployment configured back to `FAULT_ERROR_RATE: "0.0"`. |
| `09:41:10Z` | New healthy pods (`demo-6dd9bb5856`) start up. First clean HTTP 200 response logged. Error generation stops completely. |
| `09:44:32Z` | `rate5m` drops to `0.1221`, crossing below the `0.144` page threshold. |
| `09:44:51Z` | **`DemoServiceErrorBudgetBurnRateHigh` resolves!** Prometheus transitions the rule to `inactive` (`alerts: []`), and Alertmanager clears the alert. |
| `09:46:10Z` | **`rate5m` reaches `0.00000`** (exactly 5 minutes after healthy pods began serving traffic). |
| `09:48:51Z` | `DemoServiceErrorBudgetBurnRateSlow` remains firing as the 30-minute sliding window drains toward 0.06. |

---

## 4. Root Cause

PR #7 introduced a configuration regression in `deploy/apps/demo/deployment.yaml`:

```yaml
env:
  - name: FAULT_ERROR_RATE
    value: "0.3"
```

The Axum demo service reads this environment variable at startup and returns HTTP 500 on ~30% of incoming requests to `GET /`.

---

## 5. Fix

The failure was remediated purely via GitOps using a clean `git revert`:

```bash
git checkout main
git checkout -b fix/revert-1.1.0
git revert -m 1 7916067
git push origin fix/revert-1.1.0
gh pr create --title "revert: roll back release 1.1.0"
```

After PR #8 merged, Argo CD detected the new revision `38c6642` and executed a rolling update back to `FAULT_ERROR_RATE: "0.0"`. No manual `kubectl` intervention or cluster state mutation was used.

---

## 6. Verification

1. **Argo CD Application Status**:
   `kubectl get application demo -n argocd -o yaml` verified `revision: 38c66427ff681f4d0c9e55fc9057019231833dbd` and `phase: Succeeded`.
2. **Container Configuration**:
   `kubectl get pod -n demo <pod-name> -o jsonpath='{.spec.containers[0].env}'` verified `[{"name":"FAULT_ERROR_RATE","value":"0.0"},{"name":"PORT","value":"8080"}]`.
3. **Application Logs**:
   Verified logs returned HTTP 200 with zero HTTP 500 errors after `09:41:10Z`.
4. **Prometheus Recording Rules**:
   - `job:http_requests_error_ratio:rate5m` queried at `09:46:27Z` returned `0.00000`.
5. **Alert Resolution**:
   - `/api/v1/rules` showed `DemoServiceErrorBudgetBurnRateHigh` in state `inactive`.
   - `ALERTS{alertname="DemoServiceErrorBudgetBurnRateHigh"}` cleared from Prometheus.

---

## 7. Artifacts & Visual Evidence

### Prometheus Alerts Firing
Captured at ~09:30 UTC with both alerts firing:
![Prometheus Alerts Firing](../screenshots/prometheus-alerts-firing.png)

### Alertmanager UI
Showing alert start timestamps (Ticket at `09:04:21Z`, Page at `09:22:21Z`):
![Alertmanager Firing](../screenshots/alertmanager-firing.png)

### Prometheus Error Ratio Graph
Demonstrates the short 5-minute ratio jumping rapidly to ~34%, while the 1-hour ratio gradually climbs past 14.4%:
![Prometheus Error Ratio](../screenshots/prometheus-error-ratio.png)

### Grafana SLO Dashboard
Showing the request rate, error ratio, p95 latency, and remaining error budget during the fault:
![Grafana Dashboard Firing](../screenshots/grafana-dashboard-firing.png)

---

## 8. What I Learned

1. **Multi-Window Multi-Burn-Rate alerting works as intended**:
   Requiring both a short window (`5m`) and a long window (`1h`) prevented premature alerting during transient anomalies while ensuring rapid clearance: the page alert cleared at `09:44:51 UTC` (within 3 minutes and 41 seconds of healthy pod rollout), rather than lingering for an hour until the 1-hour window drained.
2. **Lab Artifact: Ticket alert fired before page alert**:
   In a production environment with weeks of steady traffic, a 6-hour window takes longer to reach threshold than a 1-hour window. However, because synthetic traffic in this lab only started at ~08:10 UTC, the "6-hour" window only held ~40 minutes of total data when the fault began, causing the ticket alert to fire at `09:04:21 UTC` before the 1-hour window crossed threshold at `09:20:21 UTC`.
3. **Gauge Error Budget Overstatement**:
   The Grafana dashboard gauge calculates remaining budget over a 30-day window (`increase(...[30d])`). In this newly bootstrapped lab cluster with ~2.5 hours of traffic, a 34% error rate over 48 minutes depleted the entire available window's budget, reading "0% budget left". In a mature 30-day system, this incident burned only ~1.94% of monthly budget.
4. **Grafana Helm chart password regeneration**:
   Grafana restarted with a new pod replica and a new random admin password on every merge to `main`. This occurs because the community Helm chart generates an admin password at chart render time when an explicit secret or existing secret is not pinned. Every Argo CD sync triggered a recreation.
5. **Rollout scrape gap**:
   During the rolling replacement at 08:51 UTC, `sum(rate(...[1m]))` briefly dipped to 0 between 08:52:00 and 08:52:30. Prometheus `rate()` requires at least two scrapes of a metric series. When pods are replaced, old series stop and new pods begin at counter=0, producing a momentary 30-second window before the second scrape where `rate()` computes 0.

---

## 9. Open Items

1. **Pin Grafana Admin Password**: Configure `adminPassword` or `existingSecret` in the Grafana Helm values so that Argo CD renders a stable secret and does not restart Grafana on every GitOps commit.
2. **Pre-seed Prometheus Data**: For future lab exercises where gauge visualizations are demonstrated, generate synthetic traffic for several days prior, or adjust gauge window calculations to match cluster age.
