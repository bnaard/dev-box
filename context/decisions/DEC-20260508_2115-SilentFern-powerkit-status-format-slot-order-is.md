---
apiVersion: processkit.projectious.work/v2
kind: DecisionRecord
metadata:
  id: DEC-20260508_2115-SilentFern-powerkit-status-format-slot-order-is
  created: '2026-05-08T21:15:48+00:00'
spec:
  title: PowerKit status-format slot order is fixed; reorder requires schema bump
  state: accepted
  decision: The current v0.25.6 PowerKit status-format slot order (hostname / external_ip / datetime / git / aibox or whatever the rendered order is in cli/src/seed.rs::tmux_status / cli/src/tmux/status.rs after the Q3 split) is intentionally fixed. Any reordering or addition/removal of slots requires (a) a schema bump on the relevant aibox.toml customization key and (b) a paired Migration entity to migrate existing user configs. Aliases for individual slot toggles may be added without a schema bump as long as default rendering order is unchanged.
  context: "v0.25.6 PowerKit work introduced a stable status-format render path (commits 79c699e log panel + 27d1510 doctor + 27d1510 status-format helpers). The KeenBison e2e gap closure (commit 78d9fc8) added a #[ignore]-gated companion test asserting the rendered status-format string contains expected slot tokens \u2014 an implicit lock on slot order. Without an explicit decision, future contributors might reorder slots for cosmetic reasons and break either the companion test or downstream user expectations / muscle memory."
  rationale: 'Locking slot order avoids three downstream risks: (1) the H3 companion e2e test silently going green-via-substring while users see different output, (2) user dashboards and screenshots in docs going stale after a release, (3) PowerKit-aware addons that read slot positions losing positional contracts. Schema bump + Migration entity is the existing aibox cross-version mechanism for this kind of breaking change (see DEC-20260508_1515-SilentAsh and the lockfile bump in commit e0ee7bc); reusing it here keeps governance consistent.'
  alternatives:
  - option: Leave slot order undocumented (status quo)
    rejected_because: Locks-by-accident through the companion test rather than by intent; future contributors will not know the constraint exists.
  - option: Make slot order user-configurable now
    rejected_because: Out of v0.25.6 scope; expands the customization surface area before there is a stable user-facing reason to.
  consequences: "Future status-format reorder PRs must (a) update cli/src/lock.rs schema_version (or its tmux-status equivalent), (b) emit a Migration entity in context/migrations/pending/ describing the old \u2192 new order, (c) update the H3 companion test if assertions were positional, (d) update any docs-site screenshots. A reviewer can reject a slot-order PR that ships none of these four things."
  deciders:
  - TEAMMEMBER-20260422_0001-MigratedMember-thrifty-otter
  related_workitems:
  - BACK-20260508_1519-LuckyLily-code-quality-aibox-toml-seed-split
  decided_at: '2026-05-08T21:15:48+00:00'
---
