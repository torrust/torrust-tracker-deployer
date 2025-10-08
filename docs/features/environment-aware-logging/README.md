# Environment-Aware Logging Feature

This folder contains the complete documentation for the environment-aware logging feature implementation.

## 📄 Documents

### [specification.md](./specification.md)

The main feature specification including:

- Overview and problem statement
- Feature goals
- Proposed solutions (multiple approaches)
- Decision rationale
- Implementation details
- Impact analysis
- Definition of done
- Testing strategy

### [questions.md](./questions.md)

Clarifying questions that need to be answered before implementation:

- Approach selection (field vs file vs span)
- Environment identification strategy
- Log file organization
- stdout/stderr migration timeline
- Cross-environment debugging needs
- Backward compatibility considerations
- Testing requirements
- Performance implications

## 📋 Status

**Current Phase**: Phase 1 Complete ✅

**Completed**:

1. ✅ Create feature specification
2. ✅ Create questions document
3. ✅ Answer clarifying questions in `questions.md`
4. ✅ Update specification based on answers
5. ✅ Commit documentation (commit 5e36da6)
6. ✅ **Phase 1 Complete**: Fixed command spans (TestCommand now has environment field)

**Next Steps**:

1. ⏳ (Optional) Phase 2: Add environment to strategic logs (10-20 key locations)
2. ⏳ Update Logging Guide with environment field usage patterns

## 🎯 Quick Summary

The application is multi-environment capable, meaning users can deploy the tracker to multiple environments (e.g., `e2e-full`, `e2e-config`, `e2e-provision`).

Commands already use tracing spans with environment context via `#[instrument]` macro. However, analysis revealed that some commands (e.g., `TestCommand`) are missing the environment field in their span instrumentation, leading to inconsistent visibility.

**Selected Solution**: **Hybrid - Improve Visibility**

Keep the existing span-based approach and fill the gaps:

1. **Add environment to missing command spans** (e.g., `TestCommand`)
2. **Add environment field to strategic logs** at application/domain layers (10-20 key logs)
3. **Keep infrastructure layers environment-agnostic** (proper abstraction)

**What we're NOT doing** (deferred):

- ❌ Custom formatter development (not needed)
- ❌ Separate log files per environment (deferred until UI)
- ❌ E2E test utility logs (not production code)

**Implementation**: Incremental - Phase 1 (command spans) is high priority, Phase 2 (strategic logs) is optional.

## 🔗 Related Documentation

- [Logging Guide](../../contributing/logging-guide.md)
- [Development Principles](../../development-principles.md)
- [E2E Testing](../../e2e-testing.md)
