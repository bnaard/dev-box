# aibox Documentation Site

This directory contains the public aibox documentation site. The site is built
with [Docusaurus](https://docusaurus.io/) and is published to GitHub Pages.

## Installation

```bash
npm install
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
npm run deploy
```

If you are using GitHub pages for hosting, this command is a convenient way to build the website and push to the `gh-pages` branch.
