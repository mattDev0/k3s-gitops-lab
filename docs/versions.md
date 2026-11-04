# Pinned Versions

This document records the exact versions of toolchains, dependencies, container base images, GitHub Actions, and utilities pinned in this repository.

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
- `fastrand`: `2.5.0` (deterministic random generation for fault injection)
- `tower`: `0.5.3` (service utilities for integration testing)

## Container Images

- **Builder Image**: `rust:1.99.1-slim-bookworm` (`sha256:ed2c95ca377d6571b124b05e75e01207973ded639dce7d5542eb7bcd797e32aa`)
- **Runtime Base Image**: `debian:12.9-slim` (`sha256:12c396bd585df7ec21d5679bb6a83d4878bc4415ce926c9e5ea6426d23c60bdc`)

## Tooling & CI Utilities

- **kubeconform**: `v0.6.7` (strict schema validation of Kubernetes manifests)
- **Trivy**: used via `aquasecurity/trivy-action@v0.36.0`

## GitHub Actions

- `actions/checkout@v7`
- `dtolnay/rust-toolchain@stable`
- `Swatinem/rust-cache@v2`
- `docker/setup-buildx-action@v4`
- `docker/build-push-action@v7`
- `docker/login-action@v4`
- `aquasecurity/trivy-action@v0.36.0`
