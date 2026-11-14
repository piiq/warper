---
name: upstream-change-analysis
description: Analyze and port changes from the original warpdotdev/warp repository into Warper. Use when comparing this fork with upstream, choosing changes to port, assessing a specific fix, or porting supplied commits or ranges.
---

# Upstream change analysis

Find changes in the original `warpdotdev/warp` repository that are worth porting into `piiq/warper`. Explain what each change does, establish its relevance to the current fork, and recommend how to port it. Implement the selected ports when the user requests implementation.

Read `AGENTS.md`, `README.md`, and the relevant current specs listed in `specs/README.md`. The maintainer's request and current Warper code determine relevance. Bug fixes, usability, performance, platform support, and maintenance improvements can all justify a port. Historical audits and archived specs do not limit new work the maintainer wants.

Check changes for dependencies on removed built-in AI, diff review, and Warp-hosted services. Extract useful changes from those dependencies where possible. Reintroducing a removed feature requires an explicit product decision.

## Establish the source

Inspect configured remotes and the current worktree. The fork's `origin` may point to `piiq/warper`; use `warpdotdev/warp` as the source for this workflow. Use the revision or range the user supplied. For an audit against the current upstream head, fetch into `FETCH_HEAD` and immediately record its SHA:

```bash
git remote -v
git fetch --no-tags https://github.com/warpdotdev/warp.git refs/heads/master
git rev-parse FETCH_HEAD
```

Use the recorded SHA throughout the audit. Preserve configured remotes and branch state. Do not create an upstream remote, branch, tag, or remote-tracking ref for the analysis. Report a failed refresh and identify the revision actually inspected.

Establish the shared history and candidate commits:

```bash
git merge-base HEAD <upstream-sha>
git log --oneline --left-right HEAD...<upstream-sha>
git show --stat --oneline --no-renames <commit>
git show --format=fuller --no-renames <commit>
```

Compare each candidate with the current local implementation. Account for changes that Warper has already ported, implemented differently, or removed.

## Establish what upstream changed

Read the patch, relevant surrounding code, and tests. For a commit with a PR number, inspect public metadata and linked issues when available:

```bash
gh pr view <number> --repo warpdotdev/warp --json number,title,body,url,closingIssuesReferences
gh issue view <number> --repo warpdotdev/warp --json number,title,body,url
```

Explain the behavior or failure that motivated upstream's change. Identify unavailable metadata and label inferences drawn from the diff or tests. Keep upstream's reason separate from the reason to port the change into Warper.

## Prove local relevance

For each candidate, establish:

1. What upstream changed and why, supported by the diff, tests, commit message, or public PR/issue.
2. Which current Warper path is affected, with file and line references. Resolve questions the repository can answer by reading the code.
3. What concrete benefit the port provides to Warper, including requested improvements to existing behavior.
4. Which new dependencies, background tasks, network calls, or product behavior the patch introduces.
5. Whether the patch can be ported directly or needs adaptation to Warper's implementation.
6. How to validate the changed behavior and adjacent workflows.

Check evidence appropriate to the change:

- For dependency changes, read the exact package versions in `Cargo.toml` and `Cargo.lock` and identify relevant usage.
- For terminal protocol changes, trace the parser, model handler, rendering or event behavior, and applicable flags.
- For third-party CLI-agent integration, check the existing session/listener path and the specific agent behavior involved.
- For platform changes, check the configured build targets and local platform code.
- For startup changes, inspect registration, process creation, watchers, timers, configuration reads, and network activity.

A security claim needs the affected local dependency or code path, reachable input, and concrete failure or exposure. An advisory title or version number alone does not establish that Warper is affected.

## Plan the port and validation

For an implementation request, apply the selected upstream changes to the current Warper code, adapt them where needed, and run the relevant checks. An audit request ends with the porting report.

Identify the behavior that must remain correct before implementing a port. For process spawning, pane/session creation, shell startup, file editing, or persistence changes, state the invariant the change relies on and check the affected creation and restoration flows.

Choose validation for the paths touched. Terminal/session changes may need coverage for new tabs, split panes, launch configs, cross-tab sequencing, and startup directories. Persistence changes need the relevant restore, migration, and new-object checks. A security fix needs the failure case and a benign retained workflow.

Use a regression test that fails before the fix when practical. Prefer a focused local port when the upstream patch mixes the useful change with unrelated work. Report required validation that cannot run in the current environment. Keep current terminal data and user configuration usable.

## Report a decision

Use these decisions:

- `Port`: the patch fits the requested scope and current code.
- `Port manually`: extract the relevant change from a patch that mixes in unrelated work.
- `Defer`: missing evidence, an unresolved product decision, dependencies, or unavailable validation prevents a recommendation to implement; state what is missing.
- `Skip`: the change is already present, the affected path is absent, the benefit does not fit the request, or the patch depends on removed functionality that cannot be separated.

For each candidate, report the commit hash and title, upstream reason and evidence, current Warper evidence, local benefit, decision, porting notes, and validation. Use a compact table or grouped list. Put recommended ports first and explain prerequisites or ordering when they matter.

Distinguish tests run with observed results from planned or unavailable checks. Create audit or spec files when the user requests them. Record skipped commits in the audit instead of creating individual specs for them. Run `rumdl check` on changed Markdown files.
