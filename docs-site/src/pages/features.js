import React from 'react';
import Layout from '@theme/Layout';

export default function Features() {
  return (
    <Layout title="Features" description="aibox features overview">
      <div className="container margin-vert--lg">
        <h1>Features</h1>

        <h2>Single Project Contract</h2>
        <p><code>aibox.toml</code> is the source of truth for the workspace: base image, container identity, addons, AI harnesses, theme, layout, runtime thresholds, and processkit source/version.</p>

        <h2>Standard Devcontainer Output</h2>
        <p>aibox generates Dockerfile, Compose, override, and devcontainer JSON files. The output is managed, but it remains readable and compatible with Docker, Podman, OrbStack, and VS Code Dev Containers.</p>

        <h2>Composable Addons</h2>
        <p>Language runtimes, AI CLIs, git tools, preview utilities, documentation frameworks, and infrastructure tools are selected through addons instead of being forced into every running container.</p>

        <h2>processkit Context Integration</h2>
        <p>processkit owns skills, processes, schemas, state machines, packages, and the canonical AGENTS.md template. aibox pins, installs, and updates that content under <code>context/</code>.</p>

        <h2>Provider-Neutral AI Harnesses</h2>
        <p>Claude Code, Codex/OpenAI, Aider, Gemini, Mistral, GitHub Copilot, Continue, Cursor registration, and related MCP configuration are selected declaratively. Provider-specific files stay thin.</p>

        <h2>Runtime Operations</h2>
        <p><code>aibox get runtime --resources</code> and <code>aibox doctor</code> report memory pressure, OOM kill counters, process counts, generated Compose posture, and selected runtime settings.</p>

        <h2>Migration System</h2>
        <p>When generated content changes, aibox preserves local edits, keeps upstream snapshots, and emits migration documents for changes that need human or agent review.</p>
      </div>
    </Layout>
  );
}
