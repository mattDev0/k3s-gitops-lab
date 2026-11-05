# Pinned Versions

This document records the exact versions of toolchains, dependencies, container base images, cluster infrastructure components, GitHub Actions, and utilities pinned in this repository.

## Rust Toolchain & Dependencies

- **Rust Toolchain**: `1.99.1` (installed via rustup on Ubuntu 24.04 LTS `x86_64`)
- **Cargo**: `1.99.1`

### Application Crates (`app/Cargo.lock`)

- `axum`: `0.8.9` (web framework)
- `tokio`: `1.54.1` (async runtime)
- `prometheus`: `0.14.0` (metrics instrumentation and text encoder)
- `serde`: `1.0.229` (serialization)
- `serde_json`: `1.0.154` (JSON serialization)
- `chrono`: `0.4.45` (timestamps for JSON logging)
- `fastrand`: `2.5.0` (fast non-cryptographic random numbers for fault injection)
- `tower`: `0.5.3` (service utilities for integration testing)

## Container Images

- **Builder Image**: `rust:1.99.1-slim-bookworm` (`sha256:ed2c95ca377d6571b124b05e75e01207973ded639dce7d5542eb7bcd797e32aa`)
- **Runtime Base Image**: `gcr.io/distroless/cc-debian12:nonroot`, pinned by digest in the Dockerfile (`sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f`)
- **Demo Service Image**: `ghcr.io/mattdev0/k3s-gitops-lab-demo:1.0.0` (`sha256:43b18242ba9b8cb5594d85e1a698de5e5e3a49dcc4dc1573d4c62d117c2c5705`)
- **Synthetic Load Generator Base Image**: `curlimages/curl:8.12.1` (`sha256:94e9e444bcba979c2ea12e27ae39bee4cd10bc7041a472c4727a558e213744e6`)
- **Prometheus Tooling Image (CI)**: `prom/prometheus:v3.15.1`

## Cluster & GitOps Infrastructure

- **K3s Kubernetes**: `v1.37.2+k3s1` (installed with Traefik and ServiceLB disabled)
- **Argo CD**: `v3.6.1` (installed via official `install.yaml`, Dex disabled)
- **kube-prometheus-stack**: `92.18.0` (pinned Helm chart from `prometheus-community`)
- **Prometheus**: `v3.15.1`
- **Promtool**: `v3.15.1` (installed in `~/.local/bin/promtool`; CI uses the `prom/prometheus:v3.15.1` image)
- **Helm**: `v4.3.1` (installed in `~/.local/bin/helm`)
- **Kubeconform**: `v0.6.7` (installed in `~/.local/bin/kubeconform`; CI tarball verified against SHA-256 `95f14e87aa28c09d5941f11bd024c1d02fdc0303ccaa23f61cef67bc92619d73`)
- **CRD schemas for kubeconform**: `datreeio/CRDs-catalog` pinned to commit `6ab3493622cd387923a2d0da5234f53e80e1bd28` (Argo CD `Application`, `AppProject`, Prometheus Operator `ServiceMonitor`, `PrometheusRule`)
- **Trivy**: used via `aquasecurity/trivy-action@v0.36.0`

## GitHub Actions

- `actions/checkout@v7`
- `dtolnay/rust-toolchain@master` with `toolchain: 1.99.1` (same version as the builder image)
- `Swatinem/rust-cache@v2`
- `docker/setup-buildx-action@v4`
- `docker/build-push-action@v7`
- `docker/login-action@v4`
- `aquasecurity/trivy-action@v0.36.0`

## Infrastructure as Code (Terraform)

- **Terraform CLI**: `1.10.5`
- **Linode Provider (`linode/linode`)**: `4.9.0` (pinned in `infra/terraform/versions.tf` and `.terraform.lock.hcl`)

