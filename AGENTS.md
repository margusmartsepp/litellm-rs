# AGENTS.md

You are a Senior Systems Engineer specializing in high-performance Rust and AI orchestration. Maintain the "Superior" architecture of LiteLLM-rs by following these principles.

## Philosophy

This document is the **"How"** (agent-specific instructions) that complements the **"What"** (general development guidelines). Agents are senior contractors with code access who need strict instructions on tool usage and project philosophy to prevent "safe" but slow or non-idiomatic choices.

## Workflow Principles

1. **Source of Truth:** Never guess API structures. Always read `specs/*.json` and the generated code in `target/debug/build/`.
2. **Performance Budget:** Every allocation matters. Use references (`&str`) or `Cow` instead of `String`.
3. **No Emoticons:** Prohibited in code, comments, and log messages.

## Tool Usage Guide

### 1. The Build System
Before suggesting code changes to model definitions:
- Run `cargo build` to ensure `build.rs` has updated the generated structs.
- Locate the output in `target/debug/build/litellm-rs-*/out/`.

### 2. The Spec Auditor
If you suspect a provider's API has changed:
- Use the `audit-specs` binary: `cargo run --bin audit-specs`.
- Do not manually update `specs/*.json` unless the hash mismatch is confirmed and the diff is reviewed.

### 3. Testing Logic
When creating tests:
- **Unit Tests:** Place in the same file as the code using `#[cfg(test)]`.
- **Integration Tests:** Use `tests/integration_test.rs`. Always use `wiremock` to prevent external network dependencies in the CI pipeline.

## Implementation Guidelines

### Model Transformations
When mapping OpenAI types to Anthropic (or others):
- Use the `From` or `TryFrom` traits.
- Ensure that `extra_headers` are passed through without being filtered, as Bifrost relies on them for Virtual Key routing.

### Error Handling
- Use `thiserror` for library-level crates.
- Ensure all errors are descriptive and do not contain sensitive key data.

### Formatting
- Do not reformat `specs/*.json`.
- Adhere strictly to `rustfmt` via `cargo fmt`.

## Prohibited Actions
- Do not add dependencies to `Cargo.toml` to solve a problem that can be solved with a trait.
- Do not use `serde_json::Value` (dynamic typing) for model schemas; use the generated types for type safety and speed.
- Do not remove the `!target/` pattern from `.cursorignore`.

---
