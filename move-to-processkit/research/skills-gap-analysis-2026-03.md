# Skills Gap Analysis — aibox vs. Ecosystem (March 2026)

**Date:** 2026-03-26
**Purpose:** Compare aibox's 84 curated skills against the broader AI coding agent ecosystem to identify gaps and expansion opportunities.

---

## 1. Methodology

This analysis compares aibox's curated skill catalog against:

1. **Competing platforms:** Cursor Rules, Windsurf Rules, GitHub Copilot Skills, Cline Custom Instructions, Continue.dev
2. **Community collections:** VoltAgent awesome-agent-skills (1,000+ skills), awesome-claude-skills (multiple repos), awesome-copilot, OpenClaw/ClawHub (13,700+ skills), SkillsMP (97,000+ skills)
3. **Developer surveys:** Stack Overflow 2025, JetBrains State of Developer Ecosystem 2025, Anthropic 2026 Agentic Coding Trends Report
4. **Market data:** SkillsMP install counts, GitHub star counts, marketplace popularity rankings

---

## 2. aibox Current Skill Inventory (84 skills, 14 categories)

| Category | Count | Skills |
|----------|-------|--------|
| Process | 9 | backlog-context, decisions-adr, standup-context, release-semver, incident-response, retrospective, agent-management, estimation-planning, postmortem-writing |
| Development | 11 | code-review, testing-strategy, refactoring, documentation, debugging, git-workflow, tdd-workflow, integration-testing, error-handling, dependency-management, code-generation |
| Language | 7 | python-best-practices, rust-conventions, typescript-patterns, go-conventions, java-patterns, sql-style-guide, latex-authoring |
| Infrastructure | 10 | dockerfile-review, ci-cd-setup, kubernetes-basics, dns-networking, terraform-basics, container-orchestration, linux-administration, shell-scripting, dependency-audit, secret-management |
| Architecture | 4 | software-architecture, event-driven-architecture, domain-driven-design, system-design |
| Design | 7 | excalidraw, frontend-design, infographics, logo-design, tailwind, pixijs-gamedev, mobile-app-design |
| Data | 5 | data-science, data-pipeline, data-visualization, feature-engineering, data-quality |
| AI/ML | 6 | ai-fundamentals, rag-engineering, prompt-engineering, llm-evaluation, embedding-vectordb, ml-pipeline |
| API | 4 | api-design, graphql-patterns, grpc-protobuf, webhook-integration |
| Security | 5 | dependency-audit, secret-management, auth-patterns, secure-coding, threat-modeling |
| Observability | 4 | logging-strategy, metrics-monitoring, distributed-tracing, alerting-oncall |
| Database | 4 | sql-patterns, database-modeling, nosql-patterns, database-migration |
| Performance | 4 | performance-profiling, caching-strategies, concurrency-patterns, load-testing |
| Framework | 5 | reflex-python, fastapi-patterns, pandas-polars, flutter-development, seo-optimization |

---

## 3. Ecosystem Skill Categories (Cross-Platform Synthesis)

Based on analysis of major marketplaces and community repositories, the broader ecosystem organizes skills into these categories (with estimated demand level):

| Ecosystem Category | Demand | aibox Coverage | Gap Level |
|-------------------|--------|----------------|-----------|
| Frontend / React / Next.js | Very High | Partial (frontend-design, tailwind) | Medium |
| Backend frameworks (Express, Django, Rails, Spring) | High | Partial (fastapi-patterns, java-patterns) | High |
| Testing & QA | High | Good (5 testing-related skills) | Low-Medium |
| DevOps & Infrastructure | High | Good (10 infra skills) | Low |
| Security | High | Good (5 skills) | Low-Medium |
| AI/ML & LLM | High | Strong (6 skills) | Low |
| Data & Analytics | Medium-High | Good (5 skills) | Low-Medium |
| API Design & Integration | Medium | Good (4 skills) | Low |
| Cloud Providers (AWS/GCP/Azure) | High | **None** | **Critical** |
| Browser Automation & Web Scraping | High (emerging) | **None** | **High** |
| Mobile Development (React Native, Swift, Kotlin) | Medium-High | Partial (flutter, mobile-app-design) | Medium |
| Documentation & Technical Writing | Medium | Partial (documentation skill) | Low-Medium |
| Monorepo Management | Medium (growing) | **None** | **Medium** |
| Accessibility (WCAG) | Medium | Partial (in frontend-design) | Medium |
| Internationalization (i18n/l10n) | Medium | **None** | **Medium** |
| Migration & Modernization | Medium | **None** | **High** |
| Video/Media Generation | Medium (niche) | **None** | Low |
| Workflow Automation | Medium | **None** | **Medium** |
| Package/Library Publishing | Medium | Partial (release-semver) | Low-Medium |
| Process Enforcement & Review | Medium | Good (9 process skills) | Low |
| Agent Orchestration | Medium (growing) | Partial (agent-management) | Low-Medium |

---

## 4. Competitive Platform Comparison

### 4.1 Cursor Rules

Cursor uses `.cursorrules` files (system-message prompts) and an official marketplace (Feb 2026) with partner integrations (Amplitude, AWS, Figma). Common rule categories:

- **Code style and formatting** (nearly universal)
- **Programming paradigms** (functional/declarative preference)
- **AI behavior guidance** ("don't apologize, fix errors")
- **Framework-specific rules** (React, Next.js, Vue, Svelte dominate)
- **Testing requirements** (mandatory CI, automated testing)

**aibox advantage:** Deeper, structured skills with reference materials vs. Cursor's flat instruction files.
**aibox gap:** No AI behavior guidance skills; no framework-specific skills for React, Next.js, Vue, Svelte, Django, Rails, Express.

### 4.2 GitHub Copilot Skills

GitHub Copilot adopted the SKILL.md format. Skills stored in `.github/skills/` (project) or `~/.copilot/skills/` (personal). The `awesome-copilot` repository organizes skills by:

- Code quality and review
- Testing and CI/CD
- Documentation
- Security
- Architecture
- Language-specific patterns

**aibox advantage:** Significantly more skills with deeper reference materials; covers more domains.
**aibox gap:** Copilot skills include GitHub-specific integrations (Issues, PRs, Actions workflows) that aibox lacks.

### 4.3 Windsurf Rules

Windsurf uses `.windsurfrules` (global) and `.windsurf/rules/` (workspace). Rules are pattern-matched to file types. Total limit: 12,000 characters combined.

**aibox advantage:** No character limit constraints; progressive disclosure model is superior.
**aibox gap:** Windsurf's file-pattern matching (auto-activate rules for `*.tsx` vs `*.py`) is a UX pattern aibox could adopt.

### 4.4 Community Collections

| Repository | Skills | Notable Categories Not in aibox |
|-----------|--------|-------------------------------|
| VoltAgent/awesome-agent-skills | 1,000+ | Browser automation, web scraping, social media, legal workflows, video processing |
| awesome-claude-skills (multiple) | 500+ | Memory management, context optimization, file/document processing, Firecrawl web access |
| OpenClaw/ClawHub | 13,700+ | Music generation, materials simulation, VMware operations, product development |
| SkillsMP | 97,000+ | Heavily frontend-skewed; 46% duplicates per HuggingFace analysis |

---

## 5. Most Popular Skills in the Ecosystem (by installs, March 2026)

| Skill | Installs | Category | In aibox? |
|-------|----------|----------|-----------|
| find-skills (meta-discovery) | 418,600 | Meta | No (N/A) |
| vercel-react-best-practices | 176,400 | Framework | **No** |
| web-design-guidelines | 137,000 | Design | Partial |
| remotion-best-practices | 126,000 | Media/Video | **No** |
| frontend-design (Anthropic) | 124,100-277,000 | Design | Yes (similar) |
| firecrawl (web data access) | High | Integration | **No** |
| agent-browser (browser control) | High | Automation | **No** |
| shannon (pen testing) | Growing | Security | **No** |

**Key insight:** Frontend/React skills dominate installs. Infrastructure, data, and security skills are vastly underrepresented in the ecosystem but are high-demand gaps -- exactly where aibox already has strength.

---

## 6. Tasks Developers Commonly Delegate to AI Agents

Based on Stack Overflow 2025, JetBrains 2025, and Anthropic 2026 surveys:

| Task Category | Delegation Rate | aibox Coverage |
|--------------|----------------|----------------|
| Writing boilerplate/scaffolding code | Very High | Partial (code-generation) |
| Writing tests | Very High | Good (testing-strategy, tdd-workflow, integration-testing) |
| Bug fixing and debugging | High | Good (debugging) |
| Code review | High | Good (code-review) |
| Refactoring | High | Good (refactoring) |
| Documentation writing | High | Good (documentation) |
| Understanding unfamiliar code | High | **No dedicated skill** |
| Database query writing/optimization | Medium-High | Good (sql-patterns) |
| CI/CD pipeline configuration | Medium-High | Good (ci-cd-setup) |
| Infrastructure as code | Medium | Good (terraform-basics, kubernetes-basics) |
| Security analysis of code | Medium | Good (secure-coding, threat-modeling) |
| Data analysis and visualization | Medium | Good (data-science, data-visualization) |
| API client/SDK generation | Medium | **No dedicated skill** |
| Codebase migration/modernization | Medium | **No dedicated skill** |
| Performance optimization | Medium | Good (performance-profiling, caching-strategies) |
| Regex writing and explanation | Medium | **No dedicated skill** |
| Environment setup and configuration | Medium | **No dedicated skill** |

---

## 7. Identified Gaps — Recommended New Skills

### 7.1 Critical Gaps (High demand, zero coverage)

| Proposed Skill | Category | Justification |
|---------------|----------|---------------|
| **cloud-aws** | Infrastructure | AWS is the most-used cloud platform (31% market share). Skills for S3, Lambda, IAM, EC2, CloudFormation/CDK are among the most requested in all marketplaces. |
| **cloud-gcp** | Infrastructure | GCP is the second most-used platform in AI/ML workloads. BigQuery, GKE, Cloud Run, IAM patterns. |
| **cloud-azure** | Infrastructure | Azure dominates enterprise. ARM/Bicep templates, Azure DevOps, App Service, AKS patterns. |
| **react-patterns** | Framework | React is the #1 frontend framework; react-related skills dominate marketplace install counts (176K+ for Vercel/React alone). Component patterns, hooks, Server Components, state management. |
| **nextjs-patterns** | Framework | Next.js App Router, Server Components, data fetching, middleware, caching, ISR/SSR/SSG patterns. The Vercel React skill at 176K installs demonstrates demand. |
| **codebase-migration** | Development | Upgrading frameworks, language versions, library replacements. A commonly delegated task with no dedicated aibox skill. Covers dependency upgrades, API migration guides, codemods. |
| **browser-automation** | Development | Browser Use and Playwright-based testing/scraping are among the fastest-growing agent capabilities. Covers E2E test authoring, web scraping, visual testing. |

### 7.2 High-Priority Gaps (Medium-high demand, no/minimal coverage)

| Proposed Skill | Category | Justification |
|---------------|----------|---------------|
| **django-patterns** | Framework | Django is the #2 Python web framework. URL routing, ORM patterns, DRF serializers, middleware, admin customization. |
| **express-node-patterns** | Framework | Express/Node.js is the most-used backend runtime. Middleware chains, error handling, clustering, TypeScript integration. |
| **vue-patterns** | Framework | Vue.js is the #3 frontend framework. Composition API, Pinia, Nuxt.js patterns. |
| **svelte-patterns** | Framework | SvelteKit is growing rapidly. Runes, load functions, form actions, server-side patterns. |
| **react-native-patterns** | Framework | Cross-platform mobile development. Navigation, native modules, performance, Expo patterns. |
| **accessibility-wcag** | Design | Dedicated WCAG 2.2 AA/AAA compliance skill. Currently embedded in frontend-design but warrants standalone depth. Screen readers, ARIA patterns, automated auditing, remediation. |
| **i18n-l10n** | Development | Internationalization and localization patterns. Message extraction, pluralization, RTL support, date/number formatting, library selection (i18next, FormatJS, fluent). |
| **monorepo-management** | Development | Turborepo, Nx, Bazel, Cargo workspaces, pnpm workspaces. Dependency graph management, CI optimization, shared config, versioning. AI is accelerating monorepo adoption. |
| **regex-patterns** | Development | Regex is one of the top tasks developers delegate to AI. Pattern composition, common recipes, performance pitfalls, dialect differences, testing strategies. |
| **environment-setup** | Infrastructure | Dev environment configuration: devcontainers, nix, asdf/mise, dotfiles, onboarding automation. |

### 7.3 Medium-Priority Gaps (Moderate demand, strategic value)

| Proposed Skill | Category | Justification |
|---------------|----------|---------------|
| **csharp-dotnet** | Language | C#/.NET is the #4 backend language. Async/await patterns, LINQ, minimal APIs, Entity Framework, Blazor. |
| **swift-conventions** | Language | iOS-native development. SwiftUI, Combine, async/await, package management. |
| **kotlin-patterns** | Language | Android-native and server-side Kotlin. Coroutines, Flow, Jetpack Compose, Ktor. |
| **ruby-rails** | Framework | Ruby on Rails remains significant. Convention over configuration, ActiveRecord patterns, Hotwire/Turbo. |
| **openapi-sdk-generation** | API | API client generation from OpenAPI specs. Code generators (openapi-generator, Kiota), SDK design, client patterns. |
| **web-scraping** | Data | Structured data extraction. Firecrawl integration, Scrapy patterns, anti-bot handling, data cleaning. |
| **technical-writing** | Process | Goes beyond documentation skill. RFC writing, architecture proposals, technical blog posts, release notes, changelog maintenance. |
| **github-workflows** | Infrastructure | Deep GitHub-specific skills: Actions workflow authoring, reusable workflows, composite actions, matrix strategies, OIDC. |
| **aws-cdk-pulumi** | Infrastructure | Modern IaC beyond Terraform. CDK constructs, Pulumi programs, type-safe infrastructure. |
| **serverless-patterns** | Architecture | Lambda/Cloud Functions, event-driven compute, cold starts, function composition, step functions. |
| **microservices-patterns** | Architecture | Service mesh, sidecar, circuit breaker, service discovery, API gateway, distributed transactions. Extends beyond the current event-driven-architecture skill. |
| **ai-agent-development** | AI/ML | Building AI agents with tool use, planning, memory, guardrails. LangChain/LangGraph, CrewAI, AutoGen patterns. Goes beyond current prompt-engineering skill. |
| **pen-testing** | Security | Offensive security testing. The "Shannon" skill (autonomous pen testing, 50+ vulnerability types) shows demand. OWASP ZAP, Burp Suite, automated scanning. |

### 7.4 Low-Priority / Niche Gaps

| Proposed Skill | Category | Justification |
|---------------|----------|---------------|
| **video-generation** | Design | Remotion has 126K installs. Programmatic video, motion graphics, animation pipelines. Niche but popular. |
| **pdf-document-processing** | Development | PDF parsing, generation, form filling. Common agent task but narrow scope. |
| **elasticsearch-patterns** | Database | Full-text search, aggregations, mappings, index lifecycle. Complements nosql-patterns. |
| **redis-patterns** | Database | Caching, pub/sub, streams, sorted sets. Partially covered in nosql-patterns but could warrant standalone. |
| **message-queues** | Architecture | RabbitMQ, SQS, Kafka consumer patterns. Partially covered by event-driven-architecture. |
| **compliance-gdpr** | Security | Data privacy patterns, consent management, data deletion, audit trails. Growing regulatory demand. |

---

## 8. Gap Heatmap by Category

| Category | Current | Recommended Additions | New Total |
|----------|---------|----------------------|-----------|
| Process | 9 | +1 (technical-writing) | 10 |
| Development | 11 | +5 (codebase-migration, browser-automation, i18n-l10n, monorepo-management, regex-patterns) | 16 |
| Language | 7 | +3 (csharp-dotnet, swift-conventions, kotlin-patterns) | 10 |
| Infrastructure | 10 | +4 (cloud-aws, cloud-gcp, cloud-azure, environment-setup) | 14 |
| Architecture | 4 | +2 (serverless-patterns, microservices-patterns) | 6 |
| Design | 7 | +1 (accessibility-wcag) | 8 |
| Data | 5 | +1 (web-scraping) | 6 |
| AI/ML | 6 | +1 (ai-agent-development) | 7 |
| API | 4 | +1 (openapi-sdk-generation) | 5 |
| Security | 5 | +1 (pen-testing) | 6 |
| Observability | 4 | +0 | 4 |
| Database | 4 | +0 | 4 |
| Performance | 4 | +0 | 4 |
| Framework | 5 | +7 (react-patterns, nextjs-patterns, django-patterns, express-node-patterns, vue-patterns, svelte-patterns, react-native-patterns) | 12 |
| **Total** | **84** | **+27** | **111** |

---

## 9. Prioritized Roadmap

### Phase 1 — Immediate (addresses critical gaps)
1. **react-patterns** — Highest ecosystem demand signal (176K+ installs for similar skills)
2. **nextjs-patterns** — Tightly coupled with React; App Router is the dominant paradigm
3. **cloud-aws** — Most-used cloud; no cloud coverage is a significant blind spot
4. **codebase-migration** — Top delegated task with zero coverage

### Phase 2 — Near-term (high strategic value)
5. **accessibility-wcag** — Regulatory pressure and ethical imperative; standalone skill warranted
6. **monorepo-management** — AI is accelerating monorepo adoption; growing need
7. **browser-automation** — Fastest-growing agent capability category
8. **django-patterns** — Fills Python backend gap alongside fastapi-patterns
9. **express-node-patterns** — Fills JavaScript/TypeScript backend gap

### Phase 3 — Medium-term (round out coverage)
10. **cloud-gcp** + **cloud-azure** — Complete cloud trifecta
11. **i18n-l10n** — Commonly needed, no good skills exist anywhere
12. **regex-patterns** — High delegation rate, low effort to create
13. **vue-patterns** + **svelte-patterns** — Framework breadth
14. **ai-agent-development** — Strategic for an AI-focused tool
15. **environment-setup** — Developer onboarding is a pain point

### Phase 4 — Long-term (completeness)
16-27. Remaining skills from sections 7.3 and 7.4 based on user feedback.

---

## 10. Strategic Observations

### 10.1 aibox Strengths vs. Ecosystem

- **Quality over quantity:** 84 curated skills with reference materials vs. 97,000+ marketplace skills where 46% are duplicates and 80% are "slop." This is aibox's core differentiator.
- **Deep infrastructure/backend coverage:** aibox's 10 infrastructure + 4 architecture + 4 database skills are better than most marketplace offerings. The ecosystem skews heavily frontend.
- **AI/ML depth:** 6 AI/ML skills with reference materials (RAG engineering, embedding/vectordb, ml-pipeline) are best-in-class for a curated collection.
- **Process skills:** 9 process skills (backlog, ADR, standup, retro, incident response, agent management) are unique -- almost no marketplace skills address team process.

### 10.2 aibox Weaknesses vs. Ecosystem

- **No cloud provider skills:** This is the single largest gap. AWS/GCP/Azure are fundamental to modern development and heavily represented in competing collections.
- **Framework coverage is narrow:** Only 5 framework skills (fastapi, reflex, pandas-polars, flutter, seo). The top 10 most-installed marketplace skills are all framework-specific (React, Next.js, Vue).
- **No browser/web automation:** Browser Use and Playwright-based skills are the fastest-growing category in 2026. This is an emergent gap.
- **Missing popular languages:** C#/.NET, Swift, and Kotlin have significant developer populations with no aibox coverage.

### 10.3 Ecosystem Trends to Watch

1. **Skill composability:** Skills referencing other skills and MCP servers for tool access.
2. **Process enforcement skills:** Skills that change how agents work, not just what they know (brainstorming before coding, mandatory test plans).
3. **Vision-based skills:** Using VLMs for UI review, accessibility auditing, design-to-code.
4. **Context management skills:** Memory, session handover, context window optimization.
5. **Security scanning skills:** Autonomous pen testing and vulnerability analysis (Shannon skill).

---

## Sources

- [VoltAgent awesome-agent-skills](https://github.com/VoltAgent/awesome-agent-skills) — 1,000+ cross-platform agent skills
- [awesome-claude-skills (travisvn)](https://github.com/travisvn/awesome-claude-skills) — Curated Claude Code skills
- [awesome-claude-skills (ComposioHQ)](https://github.com/ComposioHQ/awesome-claude-skills) — Claude skills directory
- [awesome-copilot (GitHub official)](https://github.com/github/awesome-copilot) — GitHub Copilot skills and instructions
- [GitHub Copilot Agent Skills docs](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/create-skills)
- [Cursor Agent Skills docs](https://cursor.com/docs/context/skills)
- [Cursor Agent Best Practices](https://cursor.com/blog/agent-best-practices)
- [PromptHub: Top Cursor Rules](https://www.prompthub.us/blog/top-cursor-rules-for-coding-agents)
- [Windsurf Docs](https://docs.windsurf.com/)
- [SkillsMP Marketplace](https://skillsmp.com/) — 97,000+ agent skills
- [OpenClaw/ClawHub](https://github.com/VoltAgent/awesome-openclaw-skills) — 13,700+ skills registry
- [AI Coding Statistics 2026](https://www.getpanto.ai/blog/ai-coding-assistant-statistics)
- [Stack Overflow Developer Survey 2025 — AI section](https://survey.stackoverflow.co/2025/ai)
- [JetBrains State of Developer Ecosystem 2025](https://blog.jetbrains.com/research/2025/10/state-of-developer-ecosystem-2025/)
- [Firecrawl: Best Claude Code Skills 2026](https://www.firecrawl.dev/blog/best-claude-code-skills)
- [Composio: Top 10 Claude Code Skills 2026](https://composio.dev/content/top-claude-skills)
- [Analytics Vidhya: Top AI Coding Assistants 2026](https://www.analyticsvidhya.com/blog/2026/03/ai-coding-assistants/)
- [Snyk: Top Claude Skills for Developers](https://snyk.io/articles/top-claude-skills-developers/)
- [10 Must-Have Skills for Claude in 2026 (Medium)](https://medium.com/@unicodeveloper/10-must-have-skills-for-claude-and-any-coding-agent-in-2026-b5451b013051)
