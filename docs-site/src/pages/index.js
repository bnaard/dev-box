import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import styles from './index.module.css';

function Hero() {
  const {siteConfig} = useDocusaurusContext();
  const heroImage = useBaseUrl('/img/layouts/layout-dev.svg');
  return (
    <header
      className={clsx('hero hero--primary', styles.heroBanner)}
      style={{
        backgroundImage: `linear-gradient(90deg, rgba(10, 10, 12, 0.92) 0%, rgba(10, 10, 12, 0.72) 48%, rgba(10, 10, 12, 0.35) 100%), url('${heroImage}')`,
      }}>
      <div className={clsx('container', styles.heroContent)}>
        <h1 className="hero__title">{siteConfig.title}</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <p className={styles.heroDescription}>
          Turn one project contract into a reproducible devcontainer, selected
          tools, provider-neutral agent entry points, and a managed terminal
          workspace.
        </p>
        <p className={styles.maturity}>
          Usable project — active development · Linux containers · macOS host
          support · Docker, Podman, and OrbStack
        </p>
        <div className={styles.buttons}>
          <Link className="button button--secondary button--lg" to="/docs/getting-started/installation">
            Get Started
          </Link>
          <Link className="button button--outline button--secondary button--lg" to="/docs/overview">
            Read Overview
          </Link>
        </div>
      </div>
    </header>
  );
}

const features = [
  {
    title: 'Declarative workspaces',
    description: 'aibox.toml declares container identity, addons, AI harnesses, theme, layout, and processkit source/version.',
  },
  {
    title: 'Standard devcontainer output',
    description: 'Generated Dockerfile, Compose, override, and devcontainer files stay inspectable and compatible with common runtimes.',
  },
  {
    title: 'Provider-neutral context',
    description: 'processkit content lands under context/ with AGENTS.md as the canonical entry point and provider files as thin pointers.',
  },
  {
    title: 'Runtime visibility',
    description: 'Doctor checks and resource snapshots expose memory pressure, process counts, and OOM signals before failures become opaque.',
  },
];

function Feature({title, description}) {
  return (
    <div className={clsx('col col--6')}>
      <div className="padding-horiz--md padding-vert--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

function Features() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {features.map((props, idx) => <Feature key={idx} {...props} />)}
        </div>
      </div>
    </section>
  );
}

function QuickStart() {
  return (
    <section className={styles.quickstart}>
      <div className="container">
        <h2>Quick Start</h2>
        <pre><code>{`curl -fsSL https://raw.githubusercontent.com/projectious-work/aibox/main/scripts/install.sh | bash
mkdir my-project && cd my-project
aibox init my-project --harness claude --addon python
aibox apply
aibox up`}</code></pre>
      </div>
    </section>
  );
}

function Boundaries() {
  return (
    <section className={styles.boundaries}>
      <div className="container">
        <h2>Useful today, explicit about tomorrow</h2>
        <div className="row">
          <div className="col col--4">
            <h3>What works</h3>
            <p>
              The maintained v0.x line generates and runs reproducible local
              AI workspaces from <code>aibox.toml</code>, including pinned
              processkit context.
            </p>
          </div>
          <div className="col col--4">
            <h3>What changes in v1</h3>
            <p>
              Aibox becomes the workspace image and deployment layer for
              existing Compose and Kubernetes targets. Processkit installation
              is delegated to processkit&apos;s versioned CLI protocol.
            </p>
          </div>
          <div className="col col--4">
            <h3>What aibox does not own</h3>
            <p>
              Aibox does not provision clusters, VMs, networks, identities, or
              cloud accounts, and it is not a general production application
              orchestrator.
            </p>
          </div>
        </div>
        <p className={styles.boundaryLinks}>
          <Link to="/docs/reference/compatibility">Compatibility and support</Link>
          {' · '}
          <Link to="https://github.com/projectious-work/aibox/issues/179">
            v1 architecture and progress
          </Link>
          {' · '}
          <Link to="https://projectious.work/">Projectious ecosystem</Link>
        </p>
      </div>
    </section>
  );
}

export default function Home() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title="Home" description={siteConfig.tagline}>
      <Hero />
      <main>
        <Features />
        <QuickStart />
        <Boundaries />
      </main>
    </Layout>
  );
}
