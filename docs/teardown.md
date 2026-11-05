# Cluster Teardown Runbook

> [!WARNING]
> This runbook is intended exclusively for Matt to execute manually before **2027-01-01** when the Linode VM credits expire.
> Automated tools and agents must **never** execute any teardown, uninstall, or deletion steps.

---

## 1. Overview & Objective

The `k3s-gitops-lab` cluster was established on a Linode standard instance (`dev-workspace`) to demonstrate GitOps delivery with Argo CD, Prometheus SLO multi-burn-rate alerting, and git-driven rollbacks. 

Once the lab goals are fully completed and before promotional credits end, the VM and firewall resources should be dismantled cleanly. The Git repository serves as the permanent, immutable engineering record of the architecture, code, configurations, incident history, and verification evidence.

---

## 2. Teardown Procedure

### Step 1: Confirm Repository Artifacts & Evidence
Before deleting the VM, verify that all visual artifacts and verification data are committed to the repository:
1. Verify screenshots in `docs/screenshots/`:
   - `prometheus-alerts-inactive.png` (Baseline alerts state)
   - `grafana-latency-and-budget.png` (Baseline p95 latency & budget)
   - `prometheus-alerts-firing.png` (Active firing alerts)
   - `alertmanager-firing.png` (Alertmanager active state & timestamps)
   - `prometheus-error-ratio.png` (Burn-rate curve & ratio trends)
   - `grafana-dashboard-firing.png` (Dashboard state during fault injection)
2. Ensure all branches, tags, and PRs are cleanly merged or pushed to `origin/main`.

---

### Step 2: Uninstall K3s on `dev-workspace`
SSH into `dev-workspace` (via Tailscale or internal IP):

```bash
ssh root@<dev-workspace-ip>
```

Run the official K3s uninstall script provided during cluster bootstrap:

```bash
/usr/local/bin/k3s-uninstall.sh
```

This script:
- Stops the `k3s` systemd service
- Removes the systemd service unit and links
- Deletes containerd runtime data and cgroups
- Cleans up `/etc/rancher/k3s`, `/var/lib/rancher/k3s`, and `/var/lib/kubelet`

Verify removal:
```bash
which k3s
# Expected: no output
systemctl status k3s
# Expected: Unit k3s.service could not be found.
```

---

### Step 3: Delete Linode Cloud Firewall and Virtual Machine

#### Resource Identifiers:
- **Linode Instance**: `dev-workspace` (ID: `109711421`)
- **Cloud Firewall**: `dev-workspace-fw` (ID: `229456920`)

#### Option A: Via Linode Cloud Manager (Web UI)
1. Navigate to **Firewalls** in the left navigation.
2. Locate `dev-workspace-fw` (ID `229456920`), click the **...** (More Actions) menu, and select **Delete**. Confirm deletion.
3. Navigate to **Linodes** in the left navigation.
4. Locate `dev-workspace` (ID `109711421`), click the **...** menu, and select **Delete**. Confirm deletion.

#### Option B: Via Linode CLI
If `linode-cli` is configured with a token on your workstation:

```bash
# Delete Cloud Firewall
linode-cli firewalls delete 229456920

# Delete Linode VM
linode-cli linodes delete 109711421
```

---

### Step 4: Final Verification
Confirm in the Linode Cloud Manager dashboard that no active Linodes or Firewalls remain associated with the project, ensuring billing against the account is halted.
