# Security Policy

## Supported versions

aibox is actively maintained. Security and correctness fixes are released on
the latest version; older minor lines are not maintained as parallel support
branches.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | No |

Upgrade to the latest release before reporting a problem that may already be
fixed. Compatibility with the pinned processkit version is documented in the
[compatibility matrix](https://projectious-work.github.io/aibox/docs/reference/compatibility).

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability or exposed secret.
Use GitHub's private vulnerability reporting form:

https://github.com/projectious-work/aibox/security/advisories/new

Include the affected aibox version, operating system and container runtime,
reproduction steps, impact, and any suggested mitigation. Remove live tokens,
private keys, personal data, and customer data from reports and attachments.

The maintainers will acknowledge a complete report within seven calendar days,
triage severity and affected versions, and coordinate disclosure after a fix is
available. Response and release timing depend on severity and reproducibility.

## Scope

Security reports may cover the Rust CLI, generated devcontainer configuration,
release binaries, published GHCR images, addon installers, and aibox-owned
credential or isolation behavior. Processkit-owned skills and checks should be
reported to the
[processkit repository](https://github.com/projectious-work/processkit/security).

The detailed trust boundaries, dependency provenance, and data-handling review
are maintained in the
[security reference](https://projectious-work.github.io/aibox/docs/reference/security).
