# Capability Matrix (Implemented & Verified)

This document provides a transparent view of the features and endpoints currently implemented in `litellm-rs`, along with their verification status and the environments used for testing.

| Feature / Endpoint | Status | Verification | Verified With |
| :--- | :--- | :--- | :--- |
| **`/v1/chat/completions`** | [x] **Production** | E2E & Integration | **LM Studio** (`gemma-4b`), **Cerebras** (`llama3.1-8b`) |
| **Streaming Support** | [x] **Production** | E2E & Integration | **LM Studio** (`gemma-4b`), **Cerebras** (`llama3.1-8b`) |
| **Tool Calling / Functions** | [x] **Production** | E2E (Local) | **LM Studio** (`gemma-4b`) |
| **Vision (Image Input)** | [x] **Production** | E2E (Local) | **LM Studio** (`gemma-4b`) |
| **Model Lifecycle** | [x] **Production** | E2E (Local) | **LM Studio** (`list`, `load`, `unload`) |
| **Bifrost Audit Gateway** | [x] **Production** | Integration | **Wiremock** (Caching, Failover, Budget) |
| **MCP Code Mode** | [x] **Production** | Integration | **Wiremock** (Header optimization) |
| **`/v1/embeddings`** | [ ] Planned | N/A | N/A |
| **`/v1/images`** | [ ] Planned | N/A | N/A |
| **Anthropic Support** | [/] Development | Integration | Wiremock (Payload Translation) |

---
*Last Updated: 2026-04-24*
