# RELEASE.md

## Release Process

### 1. Dependency Management
- **Audit First:** Before any release, run `cargo audit` to ensure no discovered CVEs in the tree.
- **Update Strategy:** Use `cargo update` to pull the latest compatible semver-matching dependencies.
- **MSRV Check:** Verify the **Minimum Supported Rust Version** hasn't been accidentally bumped by a dependency.
- **Lockfile Policy:** The `Cargo.lock` must be updated and committed. This is the **Source of Truth** for our deterministic builds.

### 2. Versioning & SemVer
We follow strict **Cargo SemVer** rules. Because **LiteLLM-rs** generates code from specs, a "Breaking Change" is often dictated by the providers (OpenAI/Anthropic).

| Change Type | Version Bump | Example |
| :--- | :--- | :--- |
| **Patch** | `0.1.x` | Bug fixes, performance optimizations, internal logic changes. |
| **Minor** | `0.x.0` | Adding support for a new model or provider; adding new `build.rs` features. |
| **Major** | `x.0.0` | Removing a provider; changing the `UniversalRequest` trait; breaking schema changes. |

### 3. Release Execution (Automation)
We use a **Tag-Triggered Pipeline**. Do not manually run `cargo publish` from your local machine.

1. **Update Manifest:** Bump the version in `Cargo.toml`.
2. **Changelog:** Update `CHANGELOG.md` using the "Unreleased" section.
3. **Commit & Tag:** ```bash
   git commit -am "release: v1.2.0"
   git tag -a v1.2.0 -m "Release v1.2.0"
   git push origin main --tags
   ```
4. **GitHub Release:** The CI will detect the tag and:
   - Build optimized binaries for Linux, macOS, and Windows.
   - Generate SHA-256 checksums for the `litellm-rs` executable.
   - Draft a GitHub Release with the artifacts attached.
5. **Review & Publish:** A QA Lead must review the "Draft Release" and checksums before clicking **Publish**. This triggers the final `cargo publish` to crates.io.

---

### Why this is "Superior" for you:
- **No Manual Publishes:** By letting the CI handle `cargo publish`, you ensure that the code on crates.io is *exactly* the code that passed your `tests/integration_test.rs`.
- **Artifact Integrity:** By generating checksums, you provide the "Scientific Audit" trail for users who download the binary directly rather than compiling from source.
- **Cargo-Native:** It uses the toolchain people expect (no `pyproject.toml` or `uv` logic), keeping the project clean for Rust-centric coding agents.

### Pro-Tip: The "Release-Plz" Option
In future, may consider adding **`release-plz`**. It automatically:
1. Detects if you've made changes that require a release.
2. Updates the version and changelog for you.
3. Opens a PR with the release ready to go.
