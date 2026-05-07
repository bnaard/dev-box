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
          Generate standard devcontainer files, selected tool addons,
          provider-neutral agent context, and a terminal workspace from one
          project contract.
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
aibox init my-project --context managed --harness claude --addon python
aibox apply
aibox up`}</code></pre>
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
      </main>
    </Layout>
  );
}
