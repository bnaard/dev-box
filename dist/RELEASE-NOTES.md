# aibox v0.34.7 — 2026-08-22

**Summary:** This patch improves Yazi file inspection with persistent preview controls and a hierarchical tabular size report. Existing projects receive the changes on normal apply; no processkit migration is required.

## Added

- Add persistent `w n` line-number and `w l` pane-width wrapping toggles for text and rich previews.
- Add explicit uppercase `J` and `K` bindings for vertical preview scrolling without changing the selected file.

## Changed

- Replace `w s` cumulative `du` output with a recursive, ls-like table whose size column is last and indented by directory depth.
- Document the complete Yazi preview-control surface in the file-preview guide and README.

## Fixed

- Share line-number and wrapping state across ordinary text and Rich-based previews and persist it across Yazi sessions.
- Ship the same preview plugin, helper, bindings, and executable permissions through both the runtime image and generated `.aibox-home` paths.

## Removed

- Remove the plain cumulative-size view that obscured file metadata and directory hierarchy.

## Upgrade notes

Run `aibox apply` to refresh managed Yazi configuration. The new preview toggle state is stored under the Yazi cache directory.

[v0.34.7]: https://github.com/projectious-work/aibox/compare/v0.34.6...v0.34.7
