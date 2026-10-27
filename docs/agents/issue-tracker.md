# Issue tracker: GitHub

Issues and specs live in [joeloverbeck/arkham-unfolded](https://github.com/joeloverbeck/arkham-unfolded/issues). GitHub Issues and pull requests are the sole active work-control plane. Local documents support those work items.

Use the `gh` CLI from this clone. Before the first write, verify the target with `gh repo view --json nameWithOwner`; use `--repo joeloverbeck/arkham-unfolded` when the target could be ambiguous. File work only in this repository.

## Conventions

- **Create:** `gh issue create --title "..." --body-file /tmp/issue.md`.
- **Read:** `gh issue view <number> --json number,title,body,labels,comments`.
- **List:** `gh issue list --state open --limit 100 --json number,title,body,labels,comments`, with appropriate label and state filters. Paginate when the task requires every matching issue.
- **Comment:** `gh issue comment <number> --body-file /tmp/comment.md`.
- **Labels:** `gh issue edit <number> --add-label "..." --remove-label "..."`, using the mapping in `triage-labels.md`.
- **Close:** post the explanation with `gh issue comment`, then use `gh issue close <number>`. Comment separately even when the issue is already closed.

Write multiline bodies to a temporary file and pass `--body-file`. Read back the affected issue, comment, or label after every write. If a write times out or its result is unclear, inspect remote state before retrying.

Publishing an issue or applying a readiness label follows the authority and feasibility rules in `CLAUDE.md` and the constitution. A draft spec or a closed dependency alone does not activate work.

## Sub-issues and dependencies

Use native GitHub relationships with `gh` 2.94 or later:

- **Parent:** `gh issue create --parent <parent> ...` or `gh issue edit <parent> --add-sub-issue <child>`.
- **Blocker:** `gh issue create --blocked-by <blocker> ...` or `gh issue edit <child> --add-blocked-by <blocker>`.
- **Open blocker count:** `gh issue view <number> --json blockedBy --jq '[.blockedBy.nodes[] | select(.state == "OPEN")] | length'`.

If the CLI lacks these flags, use the native REST endpoints with numeric issue database IDs: `POST repos/joeloverbeck/arkham-unfolded/issues/<parent>/sub_issues` with `sub_issue_id`, or `POST repos/joeloverbeck/arkham-unfolded/issues/<child>/dependencies/blocked_by` with `issue_id`. Obtain a database ID with `gh api repos/joeloverbeck/arkham-unfolded/issues/<number> --jq .id`.

Where native relationships are unavailable, put `Part of #<parent>` and `Blocked by: #<number>` at the top of the child body and maintain the parent's child task list. Check blocker states before selecting work.

## Pull requests as a triage surface

**PRs as a request surface: no.**

GitHub shares issue and PR numbers. Resolve a possible PR with `gh pr view <number>` and fall back to `gh issue view <number>`.

## Skill instructions

When a skill says "publish to the issue tracker", create a GitHub issue within the accepted scope. When it says "fetch the relevant ticket", read that GitHub issue.

## Wayfinding operations

When wayfinding is explicitly selected, use the same GitHub work-control plane:

- **Map:** one issue labelled `wayfinder:map`, with Notes, Decisions-so-far, and Fog sections.
- **Child:** a native sub-issue, or the parent/task-list fallback above, labelled `wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling`, or `wayfinder:task`.
- **Frontier:** the first open child in map order with no open blocker or assignee, within the accepted scope.
- **Claim:** `gh issue edit <number> --add-assignee @me`.
- **Resolve:** comment with the result, close the child, and add a concise result and link to the map's Decisions-so-far.

Create wayfinding labels only when that selected work requires them.
