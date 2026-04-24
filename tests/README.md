# LiteLLM-rs Testing Strategy

## Mission
Ensure deterministic, zero-copy AI orchestration with microsecond overhead. Every change must be verified against the Bifrost/LM Studio stack.

## Testing Pyramid

### 1. Unit Layer (Internal Logic)
- **Scope:** Zero-copy transformations, JSON parsing, header injection.
- **Rule:** No network, no disk I/O. Must pass in < 500ms.
- **Location:** `tests/unit/` and inline `#[cfg(test)]` modules.

### 2. Integration Layer (Contract Testing)
- **Scope:** Bifrost provisioning, API translation accuracy.
- **Tooling:** `wiremock` to emulate provider behavior.
- **Location:** `tests/integration/`
- **Target:** Verify that the SDK sends exactly what Bifrost expects.

### 3. E2E Layer (The "Superior" Audit)
- **Scope:** Real-world connectivity to Anthropic/OpenAI/LM Studio.
- **Location:** `tests/e2e/`
- **Requirement:** Requires local LM Studio instance for `chat_completion_test`.

## Agent Instructions (2026 Compliance)
- **Mocks:** When adding a new provider, you MUST add a corresponding `wiremock` scenario in `tests/integration.rs`.
- **Performance:** Any change to the transformation logic MUST be followed by `cargo bench`. If tail latency (P99) increases by > 5%, the PR is non-compliant.
- **Stability:** Use `tokio::test` for all async orchestrations.
