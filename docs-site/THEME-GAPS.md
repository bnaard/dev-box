# Brand theme gaps observed during content migration

These notes collect the consumer-side exceptions that remained after migrating
the site with the public v0.3.4 Hugo APIs first.

Reported upstream as
[brand-theme-hugo-vanilla#58](https://github.com/projectious-work/brand-theme-hugo-vanilla/issues/58).

- The legacy asciinema embeds exposed `data-controls="false"`,
  `data-fit="width"`, and (for theme recordings) `data-theme="..."`. The
  v0.3.4 `asciinema` shortcode accepts `src`, `poster`, `loop`, `autoplay`,
  `rows`, `cols`, `speed`, and `idleTimeLimit`, but has no equivalent options
  for controls visibility, fit mode, or a recording theme. Those attributes
  were omitted during migration.
- The old homepage used Font Awesome icons and Bootstrap layout classes. The
  homepage now uses the theme's `cards`, `card`, and front-matter CTA APIs;
  exact icon parity is not available because the brand theme uses its bundled
  Tabler icon set.
- The theme's timeline treatment is tied to the `changelog` section and the
  internal `example-changelog` class. Roadmap uses idiomatic Hugo child pages,
  weights, descriptions, and the public badge partial, but needs a small local
  list layout using that internal timeline class to achieve the requested visual
  relationship. A reusable public timeline/phase partial would remove this
  override.
- The theme gallery needs to render a generated JSON catalogue of many terminal
  recordings. The `asciinema` shortcode is content-only and its player envelope
  is not exposed as a public partial, so the local `themes/list.html` layout
  repeats that envelope while reusing the theme's CDN, script, card, and semantic
  classes. A public data-driven gallery or reusable asciinema partial would
  remove this layout-level HTML.
