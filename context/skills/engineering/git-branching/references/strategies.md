# Branching Strategy Catalog

Use this reference after the initial selection questions in `SKILL.md`.
The labels describe common patterns, not mandatory vendor tooling.

## Feature Branch Workflow

Create a focused branch from a shared integration branch, review it, merge
it back, and delete it. The integration branch can be `main`, `develop`, or
another explicitly named branch. Use it when work needs isolation but the
repository does not need a complete release-lifecycle model.

## GitHub Flow

Keep `main` deployable. Create a branch for a change, open a pull request,
validate and review it, merge to `main`, then deploy. It suits a single
actively supported production line and frequent deployments. GitHub describes
it as a lightweight, branch-based workflow.

Source: <https://docs.github.com/en/get-started/using-github/github-flow>

## Trunk-Based Development

Integrate into one trunk frequently, using short-lived branches only for
review and CI when necessary. Hide incomplete work with feature flags or
branch-by-abstraction. Cut a release branch only when a release needs a
brief hardening window.

Use it when CI is fast and reliable, the team can integrate daily, and long
divergence is more dangerous than incomplete work behind a control.

Source: <https://trunkbaseddevelopment.com/>

## Gitflow

Use `develop` for integration, feature branches from `develop`, a release
branch for stabilization, and hotfixes from `main`. Merge a completed release
to `main` and back to `develop`.

Use it for scheduled releases that need a dedicated stabilization period.
Avoid it for continuous delivery when its long-lived branch synchronization
outweighs the benefit.

Source: <https://www.atlassian.com/git/tutorials/comparing-workflows/gitflow-workflow/>

## GitLab Flow

Combine feature branches with `main` and, where needed, environment branches
or stable version branches. Promotion is explicit: a tested commit advances
from an upstream branch to a production or stable branch.

Use it when deployment environments or supported versions need visible,
separate promotion paths.

Source: <https://about.gitlab.com/topics/version-control/what-is-gitlab-flow/>

## Version-line integration

Maintain separate development and release-integration branches for each
supported major line. The release branch, not the development branch, owns
stable tags. Merge a tagged stable release to `main` so published history is
reachable from one canonical branch.

Use it when maintenance and next-major development must coexist, prereleases
need a separate channel, or the repository has previously mixed version-line
content and tags.

For example:

```text
v0.x-dev -> v0.x-release -> v0.x.y tag -> main
v1.x-dev -> v1.x-pre-release -> v1 prerelease tag
                              -> v1.x-release -> v1.x.y tag -> main
```

## Branch protection baseline

Protect every release authority and published-history branch. Require pull
requests, required status checks appropriate to the repository, resolved
conversations, and prohibit force-pushes and deletion. GitHub documents these
as branch-protection controls; choose review count and bypass policy according
to the team rather than copying a default blindly.

Source: <https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches>
