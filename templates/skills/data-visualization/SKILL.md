---
name: data-visualization
description: Data visualization best practices including chart selection, color accessibility, and dashboard design. Use when creating charts, designing dashboards, or reviewing data presentations.
allowed-tools: Bash(python:*) Read Write
---

# Data Visualization

## When to Use

When the user is creating charts, designing dashboards, choosing visualization types,
improving chart readability, or asks "what chart should I use for this data?". Also
applies when reviewing data presentations for clarity and accessibility.

## References

- `references/chart-selection.md` — Decision tree for chart types with when-to-use and when-NOT-to-use guidance

## Instructions

### 1. Chart Type Selection

- Match the chart to the data relationship, not to what looks impressive
- See `references/chart-selection.md` for the full decision tree
- Quick guide: comparison = bar, trend = line, distribution = histogram, relationship = scatter
- When unsure, default to bar chart (categorical) or scatter plot (continuous)
- Never use a pie chart with more than 4 slices — use a horizontal bar instead
- Avoid 3D charts entirely unless visualizing actual three-dimensional data

### 2. Color and Accessibility

- Use colorblind-friendly palettes: seaborn `"colorblind"`, viridis, cividis
- Never rely on color alone to convey meaning — add patterns, markers, or labels
- Sequential data (low-to-high): use a single-hue gradient or viridis
- Diverging data (deviation from center): use coolwarm or RdBu, centered at zero
- Categorical data: use distinct hues with sufficient contrast between adjacent items
- Limit to 7 or fewer colors; group or facet beyond that
- Test with a colorblindness simulator before sharing

### 3. Annotation Practices

- Annotate the insight, not just the data point
- Always annotate: max/min values that matter, threshold or target lines, events that explain trend changes
- Use direct labels instead of legends when there are 2-3 series
- Arrow annotations should point from the label to the data, not the reverse
- Keep annotation text concise: "Peak: 1,247 (Q3 launch)" not "The maximum value occurred in Q3"

### 4. Dashboard Layout

- Place the most important metric in the top-left (reading order)
- Use a consistent grid: 2-3 columns, cards of equal height per row
- Group related metrics visually: sales in one section, operations in another
- Include filters at the top, not buried in individual charts
- Every dashboard needs a title, date range, and data freshness indicator
- Limit to 6-8 visualizations per dashboard — more causes cognitive overload

### 5. Storytelling with Data

- Lead with the conclusion: "Revenue grew 40% in Q3" not "Here is a chart of revenue"
- Structure as: context (why we looked) -> finding (what we found) -> implication (what to do)
- Use progressive disclosure: summary first, then supporting details
- Highlight the relevant data; dim or remove the irrelevant
- Annotate the chart to guide the viewer's eye to the key insight

### 6. Responsive and Interactive Charts

- Static charts (matplotlib, seaborn): use for reports, papers, presentations
- Interactive charts (plotly, Altair, D3): use for dashboards and exploratory tools
- For interactive: add tooltips with detail, zoom for dense data, filter controls
- For static: ensure readability at print size (12pt+ fonts, 300 dpi)
- Always provide a static fallback for interactive charts (screenshot, PDF export)

### 7. Common Mistakes to Avoid

- Truncated y-axis on bar charts — bars must start at zero
- Dual y-axes — misleading scale comparison; use two panels instead
- Overplotting — too many points overlap; use transparency, jitter, or density plots
- Missing units on axes — every axis needs a label with units
- Default titles ("Figure 1") — the title should state the finding
- Legend far from the data it describes — place legends near the relevant series
- Too many decimal places — round to meaningful precision

### 8. Export Checklist

Before sharing any visualization, verify:
- Title states the finding or insight, not just the topic
- Both axes labeled with units
- Legend present or direct labels used
- Colorblind-friendly palette applied
- Saved at appropriate resolution (300 dpi for print, SVG/PDF for scalable)
- Fonts readable at final display size

## Examples

**User:** "I have monthly revenue data for 5 product lines over 2 years. What chart should I use?"
**Agent:** Recommends a multi-line chart with one line per product, using distinct
colorblind-safe colors and direct labels at the end of each line instead of a legend.
Adds a title stating the key trend ("Product A overtook B in mid-2025"), annotates
the crossover point, and uses a clean white background with light gridlines.

**User:** "This dashboard has 15 charts and stakeholders say it is overwhelming"
**Agent:** Audits the dashboard for redundancy (removes charts showing the same data
differently), groups remaining charts by business domain, moves detail charts to a
drill-down page, and keeps 6 key metrics on the main view. Adds a summary card row
at the top with KPIs and sparklines.

**User:** "Make this scatter plot more readable — the points overlap too much"
**Agent:** Reduces marker size and adds transparency (`alpha=0.3`), switches to a
hexbin or 2D density plot for the densest region, adds marginal histograms to show
distributions on each axis, and annotates outlier clusters with labels.
