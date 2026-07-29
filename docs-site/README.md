# aibox Documentation Site

This directory contains the public aibox documentation site. It uses
[Hugo](https://gohugo.io/) with the [Docsy](https://www.docsy.dev/) theme and
the [projectious.work brand system](https://github.com/projectious-work/brand).

## Installation

```bash
git submodule update --init --recursive docs-site/themes/docsy
npm --prefix docs-site ci
```

## Local Development

```bash
./scripts/maintain.sh docs-serve
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

## Build

```bash
./scripts/build-docs.sh
```

This command generates static content into `docs-site/public/`.

## Deployment

The repository release script validates release notes, compatibility metadata,
the root README and contributor release guidance, then runs a production Hugo
build before publication. It deploys the validated docs as part of the release
flow:

```bash
./scripts/maintain.sh release X.Y.Z
```

For a manual deployment:

```bash
cd ..
./scripts/maintain.sh docs-deploy --dry-run
./scripts/maintain.sh docs-deploy
```

The maintenance command builds locally and pushes the static output to the
`gh-pages` branch. The project does not use GitHub Actions for documentation
deployment. Do not use the generic `npm run deploy` path for this repository.
