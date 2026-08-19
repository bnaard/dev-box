# Version-line porting


# Version-line porting

aibox maintains the v0.x and v1.x lines in parallel. A fix landing on either
line must be reconciled with the other line when applicable.

The release gate compares both maintained branches after the recorded
enforcement baselines in `.github/version-line-port-baselines.toml`. Every
non-merge source commit must have a matching port on the target line or an
explicit not-applicable disposition. This derives obligations from Git history,
so it does not depend on labels, manually created issues, or workflow tokens.

When the equivalent change lands on the other line, add this commit trailer:

```text
Version-Line-Port: ported-from=<full source commit SHA>
```

The target line's release gate recognizes and settles the matching obligation.
When a change genuinely cannot or should not cross lines, document the reason
in the commit body and add:

```text
Version-Line-Port: not-applicable
```

Use `not-applicable` only for line-specific version metadata, generated release
artifacts, or code that does not exist on the other line. Do not use it to defer
an applicable fix.

Before publishing, run:

```sh
scripts/check-version-line-ports.sh check v0
scripts/check-version-line-ports.sh check v1
```

The release workflow automatically runs the gate for the major version being
published.


---
Source: https://projectious-work.github.io/aibox/docs/contributing/version-line-porting/index.md
