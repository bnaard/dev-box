/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    'overview',
    {
      type: 'category', label: 'Getting Started', collapsed: false,
      items: ['getting-started/installation', 'getting-started/new-project', 'getting-started/existing-project'],
    },
    {
      type: 'category', label: 'Container',
      items: ['container/base-image', 'container/configuration', 'container/runtime-operations', 'container/file-preview', 'container/audio'],
    },
    {
      type: 'category', label: 'Addons',
      items: [
        'addons/overview',
        'addons/language-runtimes',
        'addons/tool-bundles',
        'addons/documentation',
      ],
    },
    {type: 'doc', id: 'addons/latex-workflow', label: 'LaTeX & PDF Preview'},
    {
      type: 'category', label: 'AI Providers',
      items: ['providers/ai-claude', 'providers/ai-aider', 'providers/ai-gemini', 'providers/ai-openai', 'providers/ai-copilot', 'providers/ai-continue', 'providers/ai-mistral'],
    },
    {
      type: 'category', label: 'Context System',
      items: ['context/overview', 'context/process-packages', 'context/migration'],
    },
    {
      type: 'category', label: 'Migrations',
      items: ['migrations/lockfile-v0-25-6', 'migrations/zellij-eol', 'migrations/v0-to-v1-deployments'],
    },
    {
      type: 'category', label: 'Skills (via processkit)',
      items: ['skills/index'],
    },
    {
      type: 'category', label: 'Customization',
      items: [
        'customization/themes',
        {type: 'link', label: 'Theme Gallery', href: '/aibox/themes'},
        'customization/prompts',
        'customization/layouts',
        'customization/custom-themes',
        'customization/custom-prompts',
      ],
    },
    {
      type: 'category', label: 'Reference',
      items: ['reference/cli-commands', 'reference/configuration', 'reference/local-config', 'reference/security', 'reference/v1-deployment-boundaries', 'reference/compatibility', 'reference/cheatsheet'],
    },
    'roadmap',
    {
      type: 'category', label: 'Contributing',
      items: ['contributing/index', 'contributing/maintenance', 'contributing/e2e-tests'],
    },
  ],
};

export default sidebars;
