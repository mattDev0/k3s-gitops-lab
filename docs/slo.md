# Service Level Objective (SLO) & Burn-Rate Alerting

## 1. Overview & Disclaimer

This document defines the Service Level Objective (SLO), error budget mathematics, recording rules, multi-window burn-rate alerts, and operational runbooks for the **demo** service in `k3s-gitops-lab`.

> [!IMPORTANT]
> This SLO is a synthetic lab target established for educational purposes, platform verification, and automated fault-injection exercises. It is not an external SLA, commercial contract, or guarantee to end users.

---

## 2. SLO & SLI Specification

- **Service**: `demo` (HTTP API)
- **Service Level Indicator (SLI)**: Proportion of valid `GET /` HTTP requests that return non-5xx response codes.
- **Service Level Objective (SLO)**: $\ge 99.0\%$ availability over a rolling 30-day window (720 hours).
- **Monthly Error Budget**: $1 - 0.99 = 0.01$ (1.0% of total requests may return a 5xx error).

$$\text{Error Ratio} = \frac{\sum \text{rate}(http\_requests\_total\{route="/", method="GET", status=\sim"5.."\}[\Delta t])}{\sum \text{rate}(http\_requests\_total\{route="/", method="GET"\}[\Delta t])}$$

---

## 3. Multi-Window Multi-Burn-Rate Mathematics

Following the Google SRE Workbook (Chapter 5, *Alerting on SLOs*), alerts trigger based on the **burn rate** of the error budget rather than static error percentages. A burn rate of $1.0\times$ consumes exactly 100% of the 30-day error budget in 30 days (720 hours). A burn rate of $B$ consumes the entire error budget in $720 / B$ hours.

To avoid alerting on transient spikes (short bursts that quickly recover) while ensuring fast reaction times and rapid alert clearance, we employ **multi-window alerts** requiring both a long window and a short window to breach the threshold simultaneously.

| Alert Level | Target Consumption | Consumption Window | Burn Rate | Short Window ($\Delta t_s$) | Long Window ($\Delta t_l$) | Threshold ($B \times 0.01$) | Time to 100% Budget Exhaustion |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Page** (Fast) | 2% of budget | 1 hour | **$14.4\times$** | 5 minutes | 1 hour | **$0.144$ (14.4%)** | ~50 hours (2.1 days) |
| **Ticket** (Slow) | 5% of budget | 6 hours | **$6.0\times$** | 30 minutes | 6 hours | **$0.060$ (6.0%)** | 120 hours (5.0 days) |

### Page-Level Alert Logic ($14.4\times$ Burn Rate)
- **Condition**: Both the 1-hour error ratio and the 5-minute error ratio exceed $0.144$ (14.4%).
- **Duration**: `for: 2m` (avoids transient network blips).
- **Severity**: `page`.

### Ticket-Level Alert Logic ($6.0\times$ Burn Rate)
- **Condition**: Both the 6-hour error ratio and the 30-minute error ratio exceed $0.060$ (6.0%).
- **Duration**: `for: 2m`.
- **Severity**: `ticket`.

---

## 4. Recording Rules

Recording rules compute the error ratio across all four required time windows to eliminate redundant high-cardinality PromQL evaluations across alerts and dashboards:

Each window uses the same expression with a different range (`5m`, `30m`, `1h`, `6h`). The 5-minute rule:

```yaml
- record: job:http_requests_error_ratio:rate5m
  expr: >-
    (sum(rate(http_requests_total{route="/",method="GET",status=~"5.."}[5m])) or vector(0))
    /
    sum(rate(http_requests_total{route="/",method="GET"}[5m]))
```

The rules live in `deploy/apps/demo/prometheusrule.yaml`. CI extracts its `spec` and runs `promtool test rules` against it with `deploy/apps/demo/rules_test.yaml`: the page alert must fire on a 50% error series, both alerts must stay quiet at 0.5%, and the ratio must be 0 (not empty) when no 5xx has ever been returned. The `or vector(0)` is there because the app only creates the 5xx counter after its first error.

---

## 5. Alert Runbooks

### Runbook: High Burn Rate (Page)
- **Alert Name**: `DemoServiceErrorBudgetBurnRateHigh`
- **Severity**: `page`
- **Description**: The 1-hour and 5-minute error ratios both exceed 14.4% errors ($14.4\times$ burn rate). The entire monthly budget will be exhausted in ~50 hours if unchecked.
- **Triage Steps**:
  1. **Check Pod Status**:
     ```bash
     kubectl get pods -n demo -o wide
     ```
  2. **Examine Container Logs**:
     ```bash
     kubectl logs -n demo -l app=demo --tail=100
     ```
  3. **Check Recent Git Deployments & Commit History**:
     Inspect recent commits in `k3s-gitops-lab` and verify Argo CD sync status:
     ```bash
     kubectl get application demo -n argocd -o yaml
     ```
  4. **Inspect Fault Injection Configuration**:
     Verify if `FAULT_ERROR_RATE` in `deploy/apps/demo/deployment.yaml` was modified or injected:
     ```bash
     kubectl get deployment demo -n demo -o jsonpath='{.spec.template.spec.containers[0].env}'
     ```
  5. **Remediation**:
     If a release caused the regression, revert its commit on a branch and merge the revert through a pull request:
     ```bash
     git switch -c revert-bad-release origin/main
     git revert <commit-sha>
     git push -u origin revert-bad-release
     gh pr create --fill
     ```
     After the revert merges, Argo CD syncs `main` and rolls the Deployment back.

---

### Runbook: Slow Burn Rate (Ticket)
- **Alert Name**: `DemoServiceErrorBudgetBurnRateSlow`
- **Severity**: `ticket`
- **Description**: The 6-hour and 30-minute error ratios both exceed 6.0% errors ($6.0\times$ burn rate). Consuming 5% of monthly budget every 6 hours.
- **Triage Steps**:
  1. Open Grafana SLO Dashboard (via port-forward `kubectl -n monitoring port-forward svc/monitoring-grafana 3000:80`).
  2. Review the error ratio trend over the past 6 to 24 hours.
  3. Check if specific error codes (e.g., sporadic 500s vs 503s) correlate with load spikes or resource pressure:
     ```bash
     kubectl top pods -n demo
     ```
  4. File a ticket/issue for investigation during business hours.

---

## 6. Synthetic Traffic Generation

Under normal conditions, real user traffic does not exist in this isolated lab. To allow recording rules and SLI metrics to evaluate continuously without division-by-zero gaps:
- Deployment `loadgen` in namespace `loadgen` runs `curlimages/curl:8.12.1`.
- It loops `curl` against `http://demo.demo.svc:8080/` with a 0.2 s pause between requests, so at most about 5 requests per second.
- All traffic recorded under normal operation reflects this synthetic stream.

---

## 7. Notification Delivery Policy

Alertmanager is deployed with its **default receiver** without external webhooks, email endpoints, Discord bots, or notification tokens. Real alert delivery is intentionally omitted in this lab to avoid introducing external secrets into git.

The Alertmanager and Prometheus web interfaces serve as the authoritative evidence for alert states:
- Prometheus Alerts UI: `http://localhost:9090/alerts`
- Alertmanager UI: `http://localhost:9093`
- Grafana Dashboard: `http://localhost:3000` (search for "Demo Service SLO Dashboard")

The dashboard's "Remaining Error Budget" panel uses a 30-day `increase()`, but Prometheus keeps only 7 days of data, so the panel really shows the budget left over the last 7 days at most.
