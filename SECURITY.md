# Security Policy

## Supported Versions

Kiwi is in early development. Security fixes are applied to the default branch.
There are no long-term release branches yet.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you believe you have found a security issue in Kiwi, report it privately so we
can investigate and release a fix before details are public.

### Preferred: GitHub private reporting

1. Open [https://github.com/pacificnm/kiwi/security/advisories/new](https://github.com/pacificnm/kiwi/security/advisories/new)
2. Describe the vulnerability, affected components, and steps to reproduce
3. Include impact assessment if known (confidentiality, integrity, availability)

### Alternative

If private advisories are unavailable, contact the maintainers through a private
channel linked from the repository profile. Do not disclose exploit details in
public issues, pull requests, or discussions until a fix is available.

## What to Include

Help us respond quickly by providing:

- A clear description of the issue
- Steps to reproduce, or a proof of concept when safe to share privately
- Affected version or commit
- Impact (e.g. local privilege escalation, credential exposure, RCE)
- Suggested mitigation, if you have one

## Response Expectations

We aim to:

1. Acknowledge receipt within **5 business days**
2. Provide an initial assessment within **10 business days**
3. Coordinate disclosure and release a fix when appropriate

Timelines may vary for complex issues or during early project phases.

## Scope

In scope:

- The `kiwi` application and workspace crates under `crates/`
- Build and release tooling in `scripts/` and `tools/` when shipped or run as
  part of Kiwi development workflows
- Documented configuration and persistence paths (see
  [docs/repository-structure.md](docs/repository-structure.md))

Generally out of scope:

- Vulnerabilities in third-party dependencies already fixed upstream (please
  still report if our pinned version is affected)
- Issues in external tools Kiwi orchestrates (editors, `gh`, shells) unless Kiwi
  introduces the unsafe interaction
- Social engineering or physical attacks

## Safe Harbor

We support good-faith security research. We will not pursue legal action against
researchers who:

- Make a good-faith effort to avoid privacy violations, data destruction, and
  service disruption
- Report issues privately and allow reasonable time for remediation before
  public disclosure

## Security Considerations for Contributors

- Never commit secrets (`.env`, API keys, tokens). See `.env.example` for required
  local configuration only.
- MCP memory tools connect to PostgreSQL and OpenAI; treat database credentials
  and API keys as sensitive.
- Prefer `unsafe_code = "forbid"` in workspace crates; do not introduce `unsafe`
  without an ADR and security review.

## Related

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
