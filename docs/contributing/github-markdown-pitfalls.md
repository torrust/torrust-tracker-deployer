# GitHub Markdown Pitfalls

This document outlines common pitfalls and unexpected behaviors when writing Markdown documentation that will be rendered on GitHub. GitHub uses GitHub Flavored Markdown (GFM), which includes several automatic linking features that can create unintended references if not carefully considered.

## 🎯 Purpose

**GitHub Flavored Markdown features are powerful and should be used!** This guide is NOT about avoiding GitHub Markdown - it's about understanding its behavior to use it intentionally and avoid surprises.

**When to use GitHub Markdown features:**

- ✅ **DO** use `#42` when you want to reference issue/PR #42
- ✅ **DO** use `@username` when you want to mention someone
- ✅ **DO** use commit SHAs when referencing specific commits
- ✅ **DO** use `owner/repo#42` for cross-repository references

**This guide warns about UNINTENDED usage** - patterns that accidentally trigger GitHub's autolink behavior when you didn't mean to create a link.

### Common Problems

These issues are particularly problematic because:

- They create unintended links to issues, pull requests, discussions, or commits
- They may link to unrelated content, confusing readers
- They pollute the referenced entities with backlinks
- They make documentation harder to maintain and understand

## ⚠️ Critical Issue: `#NUMBER` Pattern

### The Problem

**GitHub automatically converts `#NUMBER` patterns into links to issues, pull requests, or discussions.**

GitHub uses a unified numbering system across issues, pull requests, and discussions within a repository. When you write `#42` anywhere in Markdown (including documentation files, comments, issue descriptions, or PR descriptions), GitHub automatically creates a link to entity #42.

### Common Mistakes

#### ❌ Bad: Enumerating with `#NUMBER`

```markdown
## Project Tasks

- Task #1: Set up infrastructure
- Task #2: Configure database
- Task #3: Deploy application
```

**What happens**: GitHub converts these to links:

- Task #1: Set up infrastructure ← Becomes a link to issue/PR/discussion #1
- Task #2: Configure database ← Becomes a link to issue/PR/discussion #2
- Task #3: Deploy application ← Becomes a link to issue/PR/discussion #3

This creates confusing and misleading links to potentially unrelated issues.

#### ❌ Bad: Numbered sections or items

```markdown
## Step #1: Install Dependencies

Follow step #1 carefully.

## Configuration Option #3

The option #3 controls timeout behavior.
```

**What happens**: `#1` and `#3` become links to issues/PRs.

### ✅ Solutions

Use alternative numbering formats that don't trigger GitHub's autolink behavior:

#### Solution 1: Use plain numbers without hash

```markdown
## Project Tasks

- Task 1: Set up infrastructure
- Task 2: Configure database
- Task 3: Deploy application
```

#### Solution 2: Use ordered lists (automatic numbering)

```markdown
## Project Tasks

1. Set up infrastructure
2. Configure database
3. Deploy application
```

#### Solution 3: Use alternative numbering schemes

```markdown
## Project Tasks

- Task (1): Set up infrastructure
- Task [1]: Set up infrastructure
- Task No. 1: Set up infrastructure
- Task number 1: Set up infrastructure
```

#### Solution 4: Use descriptive names instead of numbers

```markdown
## Project Tasks

- Task: Infrastructure Setup
- Task: Database Configuration
- Task: Application Deployment
```

### When `#NUMBER` Is Intentional

**Only use `#NUMBER` when you explicitly want to reference a GitHub entity:**

```markdown
## Recent Changes

- Fixed deployment bug in #42
- Implemented feature requested in #87
- See discussion in #105

Closes #42
```

This is the correct use case - creating intentional links to issues, PRs, or discussions.

## 🔗 Other GitHub Autolink Patterns

GitHub automatically converts several other patterns into links. Be aware of these to avoid unintended linking:

### Commit SHA References

**Pattern**: Any 7-40 character hex string that matches a commit SHA

```markdown
❌ Bad: "The version a5c3785 was released"
✅ Good: "The version a5c3785 (commit) was released" or use backticks: `a5c3785`
```

**What GitHub does**: Converts SHA strings into links to commits

**Solution**: Use backticks to prevent linking: `` `a5c3785` `` or add context to make it clear it's a commit reference.

### User and Team Mentions

**Pattern**: `@username` or `@organization/team-name`

```markdown
❌ Bad: "The @admin should configure this"
✅ Good: "The administrator (with @admin role) should configure this"
```

**What GitHub does**: Converts `@mentions` into links and **sends notifications** to mentioned users/teams

**Solution**: Avoid using `@` for generic role references. Use backticks if you need to show the literal syntax: `` `@username` ``

### Cross-Repository References

**Pattern**: `owner/repo#NUMBER`

```markdown
❌ Bad: "Similar to project-name/repo#15"
✅ Good: "Similar to issue 15 in project-name/repo"
```

**What GitHub does**: Creates links to issues/PRs in other repositories

**Solution**: Use descriptive text and only use this pattern when intentionally referencing another repository's issue.

### GH- Prefix Pattern

**Pattern**: `GH-NUMBER`

```markdown
❌ Bad: "Legacy ticket GH-123"
✅ Good: "Legacy ticket GH_123" or "Legacy ticket (GH-123 in old system)"
```

**What GitHub does**: Converts `GH-NUMBER` into issue/PR links

**Solution**: Use alternative separators (`GH_123`) or add context in parentheses.

## 📋 Best Practices Summary

### Do's ✅

- **Use ordered lists** (1., 2., 3.) for sequential items instead of `#NUMBER`
- **Use backticks** for literal syntax examples: `` `#42` `` shows "#42" without linking
- **Add context** when you must use these patterns: "issue #42" or "commit a5c3785"
- **Test your Markdown** by previewing it on GitHub before committing
- **Use alternative separators** when numbers are needed: `Task-1`, `Item [1]`, `Step (1)`

### Don'ts ❌

- **Don't use `#NUMBER`** for enumeration, step numbers, or general numbering
- **Don't use `@username`** for generic role references (use "administrator" instead)
- **Don't assume patterns are safe** - preview on GitHub to verify
- **Don't use hex strings** carelessly (they might match commit SHAs)

## 🔍 How to Escape These Patterns

If you absolutely must display these patterns literally without linking:

### Use Backticks (Code Formatting)

Backticks prevent autolinking:

```markdown
Use the `#NUMBER` pattern to reference issues.
Mention users with `@username` syntax.
Reference commits using `SHA` hashes like `a5c3785`.
```

### Use Backslash Escaping

Backslashes can escape some Markdown formatting:

```markdown
Use \#42 to avoid linking (may not work in all contexts)
```

**Note**: Backslash escaping doesn't work reliably for all autolink patterns. Backticks are more reliable.

## 🧪 Testing Your Documentation

Before committing documentation:

1. **Preview on GitHub**: Use the GitHub web interface to preview how your Markdown will render
2. **Check for unintended links**: Look for blue, underlined text where you didn't expect links
3. **Verify link targets**: Hover over links to see where they point
4. **Use GitHub's Markdown Preview**: Many editors have GitHub Markdown preview extensions

## 📚 Additional Resources

- [GitHub Flavored Markdown Spec](https://github.github.com/gfm/)
- [GitHub Docs: Basic Writing and Formatting Syntax](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax)
- [GitHub Docs: Autolinked References and URLs](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/autolinked-references-and-urls)

## 🎯 Quick Reference Table

| Pattern         | What GitHub Does                  | Avoid Using For           | Safe Alternative                        |
| --------------- | --------------------------------- | ------------------------- | --------------------------------------- |
| `#42`           | Links to issue/PR/discussion #42  | Enumeration, step numbers | `Task 1`, `Step (1)`, ordered lists     |
| `@user`         | Links to user, sends notification | Generic role references   | "administrator", "user role", backticks |
| `a5c3785` (SHA) | Links to commit                   | Version numbers, IDs      | Use backticks: `` `a5c3785` ``          |
| `owner/repo#42` | Links to issue in another repo    | General references        | Descriptive text                        |
| `GH-42`         | Links to issue/PR #42             | Legacy ticket IDs         | `GH_42` or add context                  |

**Note**: These patterns are perfectly valid when you want to create links! This table shows when to avoid them to prevent unintended linking.

## ⚡ For AI Agents

When generating Markdown documentation:

1. **Never use `#NUMBER`** for enumeration - use ordered lists (1., 2., 3.) or alternative formats
2. **Always check** if a pattern might trigger GitHub's autolink behavior
3. **Prefer descriptive text** over shorthand numbering schemes
4. **Use backticks** liberally for literal syntax examples
5. **Think about context**: Will this be viewed on GitHub? Then avoid autolink patterns.

Remember: GitHub's autolink features are designed to make cross-referencing easier, but they can create confusion when used unintentionally in documentation.
