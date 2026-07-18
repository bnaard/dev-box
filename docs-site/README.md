# aibox Documentation Site

This directory contains the public aibox documentation site. The site is built
with [Docusaurus](https://docusaurus.io/) and is published to GitHub Pages.

## Installation

```bash
npm ci
```

## Local Development

```bash
npm start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

## Build

```bash
npm run build
```

This command generates static content into `build/`.

## Deployment

The repository release script deploys docs as part of the release flow. For a
manual deployment:

```bash
cd ..
./scripts/maintain.sh docs-deploy --dry-run
./scripts/maintain.sh docs-deploy
```

The maintenance command builds locally and pushes the static output to the
`gh-pages` branch. The project does not use GitHub Actions for documentation
deployment. Do not use the generic `npm run deploy` path for this repository.
