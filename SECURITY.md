# SECURITY.md

## Security Policy

LiteLLM-rs is built with a **Deny-by-Default** security posture. We prioritize the protection of API credentials and the integrity of the AI orchestration pipeline.

## Supported Versions

Only the latest release on the `main` branch is supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| v1.x.x  | ✅ Yes             |
| < v1.0  | ❌ No              |

## Reporting Security Issues

If you discover a security vulnerability, please **do not** report it through public GitHub issues, discussions, or pull requests.

Instead, please report it through the **GitHub Security Advisory (GHSA)** private reporting feature for this repository. This allows our maintainers to review the issue and prepare a fix before the vulnerability is made public.

## Areas of Critical Concern

We are particularly interested in reports related to:
1. **Key Leakage:** Any scenario where provider API keys or "Virtual Keys" are accidentally logged, cached in plain text, or exposed via headers.
2. **Sandbox Escapes:** Issues where the Bifrost gateway or the Provisioner allows unauthorized local filesystem access.
3. **Spec Injection:** Vulnerabilities in the `build.rs` or `audit-specs` logic that could allow a malicious OpenAPI spec to execute code during compile-time.
4. **Dependency Poisoning:** Vulnerabilities found in our minimal crate tree that affect the binary's integrity.

## Our Commitment

* **Triage:** We will acknowledge your report within 48 hours.
* **Fix:** We aim to provide a patched version or a mitigation strategy within 10 business days for high-severity issues.
* **Disclosure:** We follow coordinated vulnerability disclosure. We will provide you with credit in the security advisory unless you prefer to remain anonymous.
