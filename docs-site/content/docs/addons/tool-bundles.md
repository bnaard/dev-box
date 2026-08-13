---
weight: 3
title: "Tool Bundles"
---

# Tool Bundle Addons

Tool bundles install infrastructure, orchestration, and cloud CLI tools.

## Supply Chain

```toml
[addons.supply-chain.tools]
gitleaks = {}
osv-scanner = {}
syft = {}
grype = {}
cosign = {}
```

This language-neutral bundle covers secret scanning, dependency and artifact
vulnerability checks, SBOM generation, and Sigstore signing. Release archives
are pinned and checked against upstream-published checksums.

## Release Validation

```toml
[addons.release.tools]
shellcheck = {}
hadolint = {}
```

The language-neutral `release` bundle validates shell scripts and Dockerfiles.
Go projects should normally select `[addons.go.release]`, which composes this
bundle with the `go-release` recipe and GoReleaser.

## Git UI

```toml
[addons.git-ui.tools]
gh = {}        # GitHub CLI
lazygit = {}   # Interactive Git TUI
```

The `git-ui` addon is optional. Select it when a project needs GitHub CLI
automation or lazygit inside the container; omit it to avoid installing those
tools. This aibox repo may enable it for maintenance workflows such as release
checks and GitHub issue/release work, but downstream projects do not need it by
default.

## Browser Visual Testing

The `browser-testing` addon provides a pinned Node-based Playwright Test stack
(`@playwright/test` plus `@axe-core/playwright`) for headless browser checks.
Playwright-managed full Chromium is enabled by default; Firefox and WebKit are
opt-in so a project can keep its image smaller when cross-engine coverage is
not required.

```toml
[addons.browser-testing.tools]
# Playwright Test and @axe-core/playwright are enabled by default.
# Optional cross-engine coverage:
firefox = { enabled = true }
webkit = { enabled = true }
```

The v0.x catalog currently couples Playwright `1.62.1` with
`@axe-core/playwright` `4.13.0` and the matching browser revisions. The addon
provides that pinned runner/browser environment; keep the derived project's
`package.json` and lockfile authoritative, and install the packages locally
when the project's package manager does not resolve the environment's global
packages. After applying the configuration, a derived project can keep its
tests and baselines beside the application, for example:

```ts
// playwright.config.ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/browser",
  use: {
    baseURL: "http://127.0.0.1:4173",
    ...devices["Desktop Chrome"],
    colorScheme: "light",
    reducedMotion: "reduce",
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
```

```ts
// tests/browser/home.spec.ts
import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("home page is keyboard reachable and accessible", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("main")).toBeVisible();
  await page.keyboard.press("Tab");
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
});
```

The example is only a starting point. The derived project owns the responsive
viewport set, focus and keyboard flows, light/dark themes, reduced-motion
behavior, accessibility assertions, and screenshot-baseline matrix. aibox
tests validate the addon installation, generated contract, browser launch, and
a minimal fixture; they do not prescribe an application's visual matrix.

## Preview and Archive Tools

```toml
[addons.preview-archive.tools]
chafa = {}
timg = {}
poppler = {}
mupdf = {}
entr = {}
p7zip = {}
resvg = {}
```

`preview-archive` contains the terminal image/PDF/SVG/archive helper binaries
used by Yazi previews and watch-mode document workflows. Keep it disabled for
lean headless projects that do not inspect media or generated documents inside
the terminal.

`preview-enhanced` layers Markdown, EPS, video, and Ghostscript support on top
of `preview-archive`.

## Data Preview

```toml
[addons.data-preview.tools]
sqlite3 = {}
csvkit = {}
```

`data-preview` adds read-only SQLite inspection and CSV, TSV, XLS, and XLSX
formatting for the generated Yazi preview plugins.

## Audio and Voice

```toml
[audio]
enabled = true
```

Audio bridging uses the internal `audio-voice` recipe for Sox, PulseAudio
client tools, and ALSA PulseAudio plugins. aibox selects this recipe
automatically when `[audio] enabled = true` and `install = true`; projects normally do not need to add
`[addons.audio-voice.tools]` manually.

## Infrastructure

```toml
[addons.infrastructure.tools]
opentofu = {}      # Infrastructure-as-code (Terraform alternative)
ansible = {}       # Configuration management
packer = {}        # Machine image builder
podman = {}        # Optional rootless container engine + Compose
```

OpenTofu defaults to 1.12.5, Packer to 1.16.0, and Ansible to 14.2.0.
OpenTofu and Packer are installed in a multi-stage builder. Ansible is installed via pip.
Podman is optional and installs the Debian-packaged rootless engine, Compose
provider, user-namespace helpers, and overlay/networking prerequisites. Nested
containers still depend on the outer runtime allowing user namespaces; FUSE
overlay is used when `/dev/fuse` is available, with Podman's normal fallback
otherwise.

## Kubernetes

```toml
[addons.kubernetes.tools]
kubectl = {}       # Kubernetes CLI
helm = {}          # Package manager
kustomize = {}     # Configuration customization
# k9s = {}         # Optional: terminal UI for Kubernetes
```

kubectl defaults to 1.36.3, Helm to 4.2.3, Kustomize to 5.8.1, and k9s to 0.51.0.
All tools are downloaded as static binaries in a multi-stage builder.

## Cloud Providers

### AWS

```toml
[addons.cloud-aws.tools]
aws-cli = {}
```

Installs the AWS CLI v2.

### Google Cloud

```toml
[addons.cloud-gcp.tools]
gcloud-cli = {}
```

Installs the Google Cloud CLI via the official APT repository.

### Azure

```toml
[addons.cloud-azure.tools]
azure-cli = {}
```

Installs the Azure CLI via pip.
