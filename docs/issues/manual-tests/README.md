# Manual Test Documentation

This directory contains detailed manual testing documentation for specific issues that require extensive E2E validation.

## Purpose

Some issues implement complex features (e.g., network segmentation, security configurations) that need comprehensive manual testing beyond automated tests. This folder stores those detailed test results.

## File Naming Convention

Files follow the format: `{issue-number}-{description}.md`

**Examples**:

- `XXX-feature-name-test-results.md` - Manual E2E testing documentation for issue #XXX

## When to Create Manual Test Documentation

Create manual test documentation when:

- The issue involves complex infrastructure changes that need manual verification
- Automated E2E tests cannot cover all scenarios
- Security features require extensive validation (positive and negative test cases)
- The implementation needs step-by-step testing procedures documented
- Test results with detailed command outputs need to be preserved for reference

## Structure of Manual Test Documents

Manual test documentation typically includes:

1. **Test Environment Details**: VM details, services deployed, configuration used
2. **Test Procedures**: Step-by-step commands to execute tests
3. **Expected Results**: What should happen for each test
4. **Actual Results**: Command outputs, logs, verification data
5. **Positive Tests**: Verify required functionality works correctly
6. **Negative Tests**: Verify security constraints prevent unauthorized access
7. **Network Topology**: Diagrams showing system architecture
8. **Security Analysis**: Impact assessment and compliance notes
9. **Conclusion**: Overall assessment and next steps

## Linking to Main Issue Specification

Manual test documentation should be linked from the main issue specification file in `docs/issues/`:

```markdown
- [x] **Test results documented**: See [manual test results](manual-tests/XXX-feature-name-test-results.md)
```

## Cleanup

When an issue is closed and cleaned up (see [docs/contributing/roadmap-issues.md](../../contributing/roadmap-issues.md#cleanup-process)), the associated manual test documentation should also be deleted:

```bash
# Remove manual test documentation for closed issues
cd docs/issues/manual-tests/
rm -f 21-*.md 22-*.md 23-*.md 24-*.md
```

## Version Control

Manual test documentation is committed to git along with the issue specification. This ensures:

- Test procedures are versioned and traceable
- Results can be referenced in future discussions
- Knowledge is preserved in git history even after cleanup
- Team members can review testing methodology

---

For more information about issue management and cleanup, see [docs/contributing/roadmap-issues.md](../../contributing/roadmap-issues.md).
