# [aibox] Expand `forge` to recognize configured GitHub SSH aliases

> Temporary issue draft. This file is not a posted GitHub issue.

## Summary

The `forge` tmux status plugin is enabled in the second status row, but it
renders nothing when the current repository uses a configured SSH alias for
GitHub instead of the literal hostname `github.com`.

## Observed situation

- The aibox layout includes `forge` in `line2-left`:
  `customization.tmux.status.line2-left = ["forge", "kubernetes", "terraform", "cloud"]`.
- Live tmux `status-format[1]` invokes:
  `aibox-powerkit-render-list left forge,kubernetes,terraform,cloud`.
- The active repository remote is:
  `git@github-bnaard:bnaard/internal.git`.
- The installed forge plugin parses `github-bnaard` as the remote host.
- Provider detection recognizes `github.com`, GitLab, Codeberg, and
  configured Gitea/Forgejo hosts, but does not recognize configured GitHub SSH
  aliases. The unknown-host path returns failure.
- `forge` has conditional presence, so the renderer correctly suppresses the
  segment and the row has no Forge content.

This is a detection/configuration gap, not a tmux row-formatting problem.

## Proposed behavior

Allow GitHub SSH aliases to be configured and treated as GitHub remotes while
keeping GitHub's canonical API endpoint and the existing `gh` CLI behavior.

For example, add a whitespace-separated option such as:

```tmux
set -g @powerkit_plugin_forge_github_hosts "github.com github-bnaard"
```

The default should continue to recognize `github.com`. When the parsed remote
host matches a configured GitHub host or alias, the plugin should:

1. classify the provider as GitHub;
2. continue extracting the owner and repository from SCP-style and SSH URLs;
3. use the canonical GitHub API/`gh` integration for issue and PR counts; and
4. leave unknown hosts hidden as they are today.

The option name could instead be `github_aliases` if that better matches the
existing configuration vocabulary, but it should support multiple aliases and
exact host matching.

## Acceptance criteria

- `git@github-bnaard:bnaard/internal.git` renders a GitHub Forge segment when
  `github-bnaard` is configured.
- `git@github.com:owner/repo.git` continues to work without extra
  configuration.
- `ssh://git@github-bnaard/owner/repo.git` is covered, if supported by the
  existing URL parser.
- An unconfigured host remains hidden rather than being mislabeled as GitHub.
- Unit or shell tests cover default GitHub detection, configured aliases,
  unknown hosts, owner/repository extraction, and the canonical `gh` count
  path.
- The option is documented alongside the other `forge` plugin options.

## Reproduction

1. Configure `forge` in the second tmux status row.
2. Open a pane whose repository remote is
   `git@github-bnaard:bnaard/internal.git`.
3. Observe that the `forge` segment is absent.
4. Compare with a repository whose remote host is literally `github.com`.

