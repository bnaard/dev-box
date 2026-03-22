---
name: seo-optimization
description: SEO optimization including on-page SEO, technical SEO, Core Web Vitals, structured data, and mobile-first indexing. Use when optimizing websites for search engines, implementing structured data, or improving page performance.
allowed-tools: Bash(curl:*) Bash(lighthouse:*) Read Write
---

# SEO Optimization

## When to Use

When the user is optimizing a website for search engines, asks about meta tags,
structured data, page speed, or says "improve SEO" or "add schema markup" or
"fix my Core Web Vitals".

## Instructions

### 1. On-Page SEO

- **Title tags**: unique per page, 50-60 characters, primary keyword near the start
- **Meta descriptions**: 120-160 characters, include a call to action, unique per page
- **Headings**: one `<h1>` per page matching the topic, logical `<h2>`-`<h6>` hierarchy
- **Alt text**: descriptive, concise (125 chars max), include keywords naturally
- **Internal links**: link to related content with descriptive anchor text (not "click here")
- **URL structure**: short, readable, hyphen-separated, include primary keyword
- **Content**: answer the user's query comprehensively; use related terms naturally

### 2. Technical SEO — Crawling and Indexing

- **robots.txt**: allow important paths, block admin/staging; place at site root
- **XML sitemap**: include all indexable pages, exclude noindex pages, submit to Search Console
- **Canonical URLs**: set `<link rel="canonical">` on every page to prevent duplicate content
- **Hreflang**: for multilingual sites, add `<link rel="alternate" hreflang="en">` per language
- **HTTP status codes**: 200 for live pages, 301 for permanent redirects, 404 for gone pages
- **Redirect chains**: no more than one hop; fix chains to point directly to the final URL
- **Index control**: use `<meta name="robots" content="noindex">` for pages that should not rank

### 3. Structured Data (JSON-LD)

- Add JSON-LD in `<script type="application/ld+json">` in the `<head>`
- Common types: `Article`, `Product`, `FAQ`, `BreadcrumbList`, `Organization`, `LocalBusiness`
- Match structured data to visible page content — do not add data not shown on the page
- Validate with Google Rich Results Test before deploying
- Use `@context: "https://schema.org"` and the most specific `@type` available
- BreadcrumbList helps search engines understand site hierarchy

### 4. Core Web Vitals

- **LCP (Largest Contentful Paint)**: target <2.5s
  - Preload the LCP image: `<link rel="preload" as="image" href="...">`
  - Use responsive images with `srcset` and appropriate sizes
  - Inline critical CSS, defer non-critical stylesheets
- **INP (Interaction to Next Paint)**: target <200ms
  - Break up long tasks (>50ms) with `requestIdleCallback` or `scheduler.yield()`
  - Minimize main thread work during interaction handlers
  - Debounce rapid input events
- **CLS (Cumulative Layout Shift)**: target <0.1
  - Set explicit `width` and `height` on images and video
  - Reserve space for ads, embeds, and late-loading content
  - Avoid inserting content above the fold after initial render
  - Use `font-display: swap` with preloaded fonts

### 5. Page Speed Optimization

- Minimize render-blocking resources: inline critical CSS, defer JS
- Compress images: use WebP/AVIF with fallbacks; lazy-load below-fold images
- Enable HTTP/2 or HTTP/3 for multiplexed requests
- Use a CDN for static assets — reduce time to first byte (TTFB)
- Minify HTML, CSS, and JS; enable Brotli/gzip compression
- Audit with Lighthouse: `npx lighthouse <url> --output=json`

### 6. Mobile-First Indexing

- Google indexes the mobile version of the page — ensure content parity with desktop
- Responsive design preferred: same URL, same HTML, CSS adapts
- Viewport meta tag: `<meta name="viewport" content="width=device-width, initial-scale=1">`
- Text readable without zooming: 16px minimum base font size
- No horizontal scrolling — content fits within the viewport
- Touch targets at least 48x48 CSS pixels with 8px spacing
- Test with Chrome DevTools device emulation and Google Mobile-Friendly Test

### 7. Internal Linking Strategy

- Link from high-authority pages to important pages that need ranking signals
- Use descriptive anchor text that reflects the target page's topic
- Create content hubs: pillar page linking to cluster pages and back
- Limit links per page to a reasonable number (under 100 for most pages)
- Fix orphan pages — every indexable page should be reachable via internal links
- Use breadcrumbs for hierarchical navigation and search engine context

### 8. Security and Trust Signals

- Serve all pages over HTTPS — required for ranking and user trust
- Set HSTS header: `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- Avoid mixed content (HTTP resources on HTTPS pages)
- Add clear authorship (E-E-A-T): author bios, about pages, contact information
- Display trust elements: reviews, certifications, security badges where relevant

## References

- `references/technical-seo-checklist.md` — Actionable checklist by category
- [Google Search Central](https://developers.google.com/search)
- [web.dev Core Web Vitals](https://web.dev/vitals/)

## Examples

**User:** "Audit the SEO of my Next.js site"
**Agent:** Checks for meta tags on key pages, verifies sitemap.xml and robots.txt
exist and are correct, validates structured data with Rich Results Test, runs
Lighthouse for Core Web Vitals, checks canonical URLs, and produces a prioritized
list of fixes sorted by expected impact.

**User:** "Add structured data to my product pages"
**Agent:** Creates a `Product` JSON-LD schema with name, description, image, price,
availability, and aggregateRating. Places it in the page `<head>`, ensures all
fields match visible content, and validates with Google's Rich Results Test.

**User:** "My LCP score is 4.5 seconds — help me fix it"
**Agent:** Identifies the LCP element (usually a hero image or heading), adds a
preload hint for the LCP image, converts it to WebP with responsive srcset, inlines
critical CSS, defers non-essential JS, and verifies improvement with Lighthouse.
