---
apiVersion: processkit.projectious.work/v2
kind: Note
metadata:
  id: NOTE-20260509_2237-VastVale-uv-image-lag-0-11-10
  created: '2026-05-09T22:37:20+00:00'
spec:
  title: "uv image lag: 0.11.10 pinned, 0.11.12 latest \u2014 SureSeal/BraveFalcon branches only reach 0.11.11"
  body: |
    During PluckyEagle drift review (2026-05-10), RELEASE-STATE.md shows uv latest at 0.11.12 while:
    - aibox.toml dogfood pin: 0.11.10
    - BraveFalcon branch (7a26c5e, v0.25.7/sureseal-uv-image-review): bumped to 0.11.11 but not merged
    - Existing workitems: BACK-20260508_1214-SureSeal + BACK-20260507_0552-BraveFalcon cover 0.11.11 step

    Next action: when merging SureSeal/BraveFalcon, bump directly to 0.11.12 (skipping 0.11.11) to avoid two image rebuilds. Verify uv 0.11.12 release notes for breaking changes before bumping. Requires updating: images/base-debian/Dockerfile, addons/languages/python.yaml, aibox.toml, relevant tests and docs.
  type: fleeting
  state: captured
  review_due: '2026-05-17'
  tags:
  - dependencies
  - uv
  - base-image
  - drift
---

During PluckyEagle drift review (2026-05-10), RELEASE-STATE.md shows uv latest at 0.11.12 while:
- aibox.toml dogfood pin: 0.11.10
- BraveFalcon branch (7a26c5e, v0.25.7/sureseal-uv-image-review): bumped to 0.11.11 but not merged
- Existing workitems: BACK-20260508_1214-SureSeal + BACK-20260507_0552-BraveFalcon cover 0.11.11 step

Next action: when merging SureSeal/BraveFalcon, bump directly to 0.11.12 (skipping 0.11.11) to avoid two image rebuilds. Verify uv 0.11.12 release notes for breaking changes before bumping. Requires updating: images/base-debian/Dockerfile, addons/languages/python.yaml, aibox.toml, relevant tests and docs.
