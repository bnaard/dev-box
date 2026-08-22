---
title: "v0.34.7 — configurable Yazi previews"
description: "Adds persistent line numbers and wrapping, explicit preview scrolling, and hierarchical tabular size reports."
date: 2026-08-22
author: "projectious.work"
tags: [release]
---

The v0.34.7 patch makes file inspection in Yazi more deliberate and useful.
Text and rich previews now share persistent line-number and pane-width wrapping
toggles, while uppercase `J` and `K` scroll the preview without moving the
selected file.

The `w s` size view is now a recursive table: the tree remains on the left,
standard file metadata occupies aligned columns, and sizes appear last with
depth indentation so directory totals are easy to compare.

[Full v0.34.7 release notes](https://github.com/projectious-work/aibox/releases/tag/v0.34.7)
