---
name: governed-pr-flow
description: Governed workflow for preparing, sorting, and submitting pull requests to remote repositories. Use when the user asks to create a branch, push commits, and open a PR with gh while preserving task-registry validation and proof.
---

# Governed PR Flow

Use this workflow to take already-implemented or staged work to a remote PR under task-registry governance.

**Project extensions:** when `PROJECT.md` exists beside this skill, load it after this file for repo-specific branch policy, base branch, CI gates, and PR templates.

## 1) Confirm PR intent and scope

- Confirm repo root (`git rev-parse --show-toplevel`) matches the intended project.
- Confirm user wants a remote PR (not only local commit).
- Read current project `AGENTS.md` and active plan/task context.

## 2) Validate governance readiness for PR

- Ensure the implementation plan has been activated:
  - `.codex/scripts/task-registry activate docs/plans/<slug>.md`
- Confirm landing and verifier proof are complete for files in scope:
  - `.codex/scripts/task-registry verify-landing --plan-id <plan_id> --changed-files <path>...`
- When registry state changed, verify receipts:
  - `.codex/scripts/task-registry verify-chain --format json`

## 3) Sort changes into coherent buckets

- Sort changes into coherent buckets by intent (feature, fix, refactor, docs/tests/governance).
- Keep each bucket minimal, reviewable, and tied to task/behavior proof.
- Exclude unrelated dirty files from the PR branch.

## 4) Stage and commit by bucket

- Stage only the files for one bucket.
- Commit with message style used by the repository.
- Repeat until all intended buckets are committed.

## 5) Prepare branch for remote

- Create or switch to a review branch.
- Confirm branch is correct and ready to publish.

## 6) Push branch and create remote PR

- Push and set upstream:
  - `git push -u origin HEAD`
- Open PR with GitHub CLI:
  - `gh pr create --title "<title>" --body "<body>"`
- Include in PR body:
  - Scope summary tied to plan/task.
  - Validation commands executed.
  - Deferred/blocked tasks with reason, if any.

## 7) Final PR handoff

- Run final report for traceability:
  - `.codex/scripts/task-registry report <plan_id>`
- Share: PR URL, commands run, outcomes, and remaining risk/deferred items.

## Example first use

- Use this skill to submit the `governed-pr-flow` skill addition itself as the first remote PR from the current branch.
