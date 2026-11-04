# Architecture & Design Decisions

This document details the architecture of the `k3s-gitops-lab` single-node environment, illustrating how code flows from git to cluster deployment and recording key architectural design decisions.

## System Architecture

```mermaid
flowchart TD
    subgraph GitHub ["GitHub (mattDev0/k3s-gitops-lab)"]
        GitRepo["Git Repository\n(main branch)"]
        GHA["GitHub Actions\nCI / Release Pipeline"]
        GHCR["GitHub Container Registry\n(ghcr.io/mattdev0/k3s-gitops-lab-demo)"]
    end

    subgraph LinodeVM ["Linode dev-workspace (8 GB VM)"]
        subgraph K3sCluster ["Single-Node K3s Cluster"]
            subgraph ArgoCDNamespace ["Namespace: argocd"]
                RootApp["Argo CD Root App\n(app-of-apps)"]
                DemoApp["Argo CD Child App\n(demo)"]
                ArgoServer["Argo CD Server\n(UI & API)"]
            end

            subgraph DemoNamespace ["Namespace: demo"]
                DemoPods["Demo Microservice Pods\n(2 Replicas, Port 8080)"]
                DemoSvc["ClusterIP Service\n(demo:8080)"]
            end
        end

        LocalPortForward["localhost Port-Forward\n(127.0.0.1:8081 / 18080)"]
    end

    GitRepo -->|"Tag Push (v*)"| GHA
    GHA -->|"Build & Push (1.0.0)"| GHCR
    GitRepo -->|"Poll / Webhook Sync"| RootApp
    RootApp -->|"Deploys & Watches"| DemoApp
    DemoApp -->|"Reconciles Manifests"| DemoNamespace
    GHCR -->|"Pulls Image (no login)"| DemoPods
    LocalPortForward -.->|"Admin Access"| ArgoServer
    LocalPortForward -.->|"Direct Probe"| DemoSvc
```

---

## Design Decisions

### 1. Why Argo CD
I selected Argo CD as the GitOps delivery engine because it operates entirely within the cluster, polling Git for the desired state and comparing it continuously against live cluster state. It provides built-in automated sync, self-healing (`selfHeal: true`), prune capabilities (`prune: true`), and clear drift visibility without requiring cluster credentials to leave the machine.

### 2. Why Traefik and ServiceLB are Disabled
The Linode host runs a strictly private network model:
- Tailscale serves private tooling (such as code-server) terminating TLS on port 443 on the tailnet interface.
- Standard K3s bundles Traefik ingress and Klipper ServiceLB, which bind to `0.0.0.0:80` and `0.0.0.0:443`.
- Disabling both via `--disable traefik --disable servicelb` prevents port binding conflicts with Tailscale and guarantees no service or port is exposed publicly on the Linode's public interface. All interaction with the cluster (Argo CD UI, demo endpoints) is strictly mediated via `kubectl port-forward` bound to `127.0.0.1`.

### 3. Why Image Tags Only Change via Pull Requests
Automated tools that write image tag bumps directly back to Git repositories blur the boundary between build artifacts and production deployment intent. By requiring container image tags to be pinned explicitly in `deploy/apps/demo/deployment.yaml` and updated only through PRs:
- Every deployment change produces a clear, auditable Git commit with a distinct author and commit message.
- Reverting a deployment (demonstrated in Stage 4) is a straightforward `git revert`.
- Continuous Delivery cannot accidentally deploy untested tags to the cluster.

### 4. What Is Deliberately Left Out
- **Public Ingress & DNS/TLS**: The lab does not serve public traffic or require Let's Encrypt certificates. It is an internal demonstration environment accessed securely over localhost port-forwards.
- **High Availability (HA) & Multi-Node**: The entire platform runs on a single Linode standard instance with 8 GB RAM and 4 vCPUs. Clustering overhead (etcd quorum, redundant control planes) is unnecessary for the scope of this learning lab and would consume valuable memory needed by Prometheus and Argo CD.
