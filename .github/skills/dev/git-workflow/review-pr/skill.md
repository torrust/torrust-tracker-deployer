---
name: review-pr
description: Review a pull request against the project's quality standards. Covers the full checklist from the PR review guide — branching, commits, code quality, testing, documentation, and template conventions — and how to provide constructive feedback. Use when reviewing a pull request, checking if a PR meets project standards, or providing feedback on changes. Triggers on "review PR", "review pull request", "check PR", "PR checklist", "code review", or "approve PR".
metadata:
  author: torrust
  version: "1.0"
---

# Review a Pull Request

## Process

1. **Read the PR review guide** — [`docs/contributing/pr-review-guide.md`](../../docs/contributing/pr-review-guide.md) is the single source of truth. Do not rely on your general knowledge; follow the checklist there.

2. **Work through each checklist item** in the guide against the actual changes in the PR:
   - Branching & Commits
   - Code Quality (DDD layers, error handling, module organization)
   - Testing
   - Documentation
   - Templates (if applicable)

3. **Check the Quick Red Flags section** — scan the diff for the listed architecture violations, error handling issues, code organization problems, and testing issues.

4. **Distinguish expected from real errors** — consult [`docs/contributing/known-issues.md`](../../docs/contributing/known-issues.md) before flagging anything as a problem.

5. **Provide feedback** following the guide's conventions:
   - **Request Changes** for blocking violations of documented standards
   - **Comment** for non-blocking suggestions
   - **Approve** only when all checklist items are satisfied

> The guide has examples of good vs. better feedback with specific references to documentation and line numbers. Follow that pattern.
