# DEVELOPMENT.md

## Development Guidelines

### Branching and Versioning
* **Main Branch:** Current development line. Breaking changes are allowed but must be documented in `docs/MIGRATION.md`.
* **Zero-Shim Policy:** Do not add backward-compatibility layers or deprecated shims. If an API is replaced, delete the old version and document the migration path.
* **Semantic Versioning:** Follow strict SemVer. Any change to the generated structs from `build.rs` that removes a field requires a Major version bump.

### Package Management (Cargo)
* **Standard Tooling:** Only use `cargo` commands. Avoid manual edits to `Cargo.toml` where `cargo add` suffices.
* **Lockfile:** `Cargo.lock` must be committed to ensure deterministic audits and builds.
* **Minimalist Dependencies:** Every crate must be justified. Prefer the Standard Library or small, focused traits over heavy external frameworks.
* **Security Audits:** Run `cargo audit` before every release. High-severity vulnerabilities in production dependencies are block-level issues.

### Code Quality and Patterns
* **No Emoticons:** Prohibited in all comments, docstrings, logs, and commit messages.
* **Error Handling:** * Use `thiserror` for library-level crates and public APIs.
  * Use `anyhow` for the CLI and proxy binary logic.
  * `unwrap()` and `expect()` are forbidden in production code; they are only permitted in tests and examples.
* **Memory Management:** Use `std::borrow::Cow` for model names and prompt content to minimize allocations during high-throughput agent loops.
* **Documentation:** All public traits and structs must be documented. Use `cargo doc` to verify the output.

### Testing Strategy
* **Framework:** Standard `cargo test` with `tokio::test` for async execution.
* **Fast and Deterministic:** Use `wiremock` for network-level integration tests to avoid external dependencies.
* **Benchmarking:** Use `criterion`. Any change affecting `src/models.rs` or transformation logic must include a benchmark report. A P99 latency increase greater than 5% is a rejected regression.

### Schema Generation and Build Process
* **Source of Truth:** `build.rs` is the authority. Do not manually edit files in generated paths.
* **Audit-Only Upgrades:** Use `cargo run --bin audit-specs` to check for provider schema updates. Do not auto-upgrade; manually verify the diff before committing new specs.

#### Audit Strategy
* **Schema Discovery Logic:**
  * **Dynamic Resolution:** The auditor resolves remote specs via `.stats.yml` where available (Stainless-style).
  * **Checksum Integrity:** Comparison is based on SHA-256 hashes of the raw bytes to ensure format changes (JSON vs YAML) are detected as content drifts.

### Logging and Observation
* **Tracing:** Use the `tracing` crate for all logging.
* **Sensitive Data:** Never log raw API keys, virtual keys, or identifiable user prompts in `INFO` or `DEBUG` logs.
* **Log Levels:** * `ERROR`: Critical system failures (e.g., Bifrost process crash).
  * `WARN`: Recoverable issues (e.g., provider fallback triggered).
  * `INFO`: Lifecycle events (e.g., new session started).
