# Decision: SDK Package Naming

## Status

Accepted

## Date

2026-02-26

## Context

The `packages/sdk` package provides a typed Rust API for deploying and managing
Torrust Tracker instances programmatically, as an alternative to the CLI. It
wraps the same application-layer command handlers used by the CLI, offering a
friendlier API for AI agents, scripts, and automation pipelines.

During a maintainer discussion on February 26, 2026, a question arose about
whether "SDK" was the right name for this package. One maintainer argued that
SDK (Software Development Kit) historically denotes something much larger — a
comprehensive suite of tools, compilers, debuggers, and frameworks for a
specific platform or operating system — and that our package is merely an API
client library. The other maintainer argued that the SDK term has undergone a
well-documented semantic evolution and is now widely accepted as the standard
term for language-specific wrappers around APIs, regardless of their size.

To resolve the question rigorously, a deep research report was commissioned
using Gemini Pro Deep Research. The full report is available at
[`docs/research/sdk-toolset-vs-api-wrapper.md`](../research/sdk-toolset-vs-api-wrapper.md).

### Key Findings from the Research

The report confirms that "SDK" carries **two legitimate and coexisting
definitions** in current industry practice:

1. **Traditional / formal definition** (IEEE/ISO standards, SWEBOK): A
   comprehensive, self-contained suite of tools — including compilers,
   debuggers, simulators, and frameworks — required to build software for a
   specific hardware platform or operating system (e.g., Android SDK, iOS SDK,
   Windows SDK).

2. **Modern / cloud-era definition**: A language-specific client library that
   wraps a remote API, abstracting authentication, HTTP handling, serialisation,
   and retry logic, so consumers can interact with a service using idiomatic
   code. This definition dominates the cloud and SaaS industry today
   (e.g., AWS SDK, Azure SDK, Stripe SDK, Firebase SDK).

The research further documents that this semantic drift was driven by major
technology vendors (Amazon, Microsoft, Google, Stripe, etc.) deliberately
rebranding their client libraries as "SDKs" to signal completeness, support,
and a polished developer experience. ISO/IEC/IEEE standardisation bodies have
not updated their definitions to reflect this shift, creating a gap between
formal vocabulary and pragmatic industry usage.

### What Our Package Is

`packages/sdk` is exactly what the modern definition describes: an idiomatic,
typed Rust API that wraps the deployer's programmatic interface. It:

- Provides a `Deployer` struct with builder-pattern initialisation
- Exposes high-level methods (`list`, `create`, `provision`, `configure`,
  `release`, `run`, `show`, `test`, `destroy`) that map to application-layer
  command handlers
- Returns `()` or purpose-built result types — no domain types leak through
  the public API (see [SDK Presentation Layer Interface Design ADR](./sdk-presentation-layer-interface-design.md))
- Ships with usage examples and is designed to be consumed as a Cargo
  dependency
- Does **not** include compilers, debuggers, build systems, or platform
  simulators

This is precisely the pattern used by AWS SDK for Rust, Azure SDK for Rust, and
other modern Rust SDKs.

## Decision

Keep the name **SDK** for `packages/sdk` and the crate
`torrust-tracker-deployer-sdk`.

The modern usage of "SDK" as a language-specific API integration library is
well-established across the industry. Renaming it to "client" or "library"
would be less immediately recognisable and would diverge from the conventions
of major cloud providers whose Rust packages our package resembles most closely
in structure and purpose.

## Consequences

### Positive

- ✅ **Industry-standard terminology**: Developers familiar with AWS SDK, Azure
  SDK, or Stripe SDK will immediately understand what this package is for
- ✅ **Discoverable naming**: "SDK" implies a polished, supported, idiomatic
  integration — setting the right expectations for consumers
- ✅ **Consistent with related ADRs**: [SDK Presentation Layer Interface Design](./sdk-presentation-layer-interface-design.md)
  and other ADRs already use this terminology throughout

### Negative

- ⚠️ **Potential terminology friction**: Developers coming from low-level
  systems or hardware backgrounds may expect an SDK to include compilers and
  build toolchains. This is mitigated by the README description and
  documentation.

### Neutral

- 📝 The package remains scoped to programmatic deployment operations; it is
  not a generic Torrust Tracker client library

## Alternatives Considered

### Option 1: Rename to `packages/client` / `torrust-tracker-deployer-client`

**Pros:**

- Technically precise under the formal IEEE/ISO definition
- Avoids any perceived overstatement

**Cons:**

- ❌ "client" is a much weaker signal of quality and support than "SDK"
- ❌ Diverges from industry conventions used by AWS, Azure, Stripe, and similar
  providers for identical types of packages
- ❌ Requires renaming all existing code, ADRs, documentation, and crate
  references

### Option 2: Rename to `packages/api` / `torrust-tracker-deployer-api`

**Pros:**

- Short and unambiguous in meaning

**Cons:**

- ❌ Conflicts with the established use of "API" to mean an interface
  specification, not an implementation
- ❌ Even more misleading than "client" — an API usually denotes what is
  exposed, not a library that consumes it

## Related Decisions

- [SDK Presentation Layer Interface Design](./sdk-presentation-layer-interface-design.md)
- [SDK Discarded — Fluent Interface](./sdk-discarded-fluent-interface.md)
- [SDK Discarded — Scoped Environment Guard](./sdk-discarded-scoped-environment-guard.md)
- [SDK Discarded — Typestate at SDK Layer](./sdk-discarded-typestate-at-sdk-layer.md)

## References

- Research report: [`docs/research/sdk-toolset-vs-api-wrapper.md`](../research/sdk-toolset-vs-api-wrapper.md)
- [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/)
- [Azure SDK for Rust](https://github.com/Azure/azure-sdk-for-rust)
- [ISO/IEC/IEEE 24765:2017 — Systems and software engineering vocabulary](https://cdn.standards.iteh.ai/samples/71952/6289cd982a154c1d8fa0b10b52e0f8a8/ISO-IEC-IEEE-24765-2017.pdf)
- [SDK vs API — AWS](https://aws.amazon.com/compare/the-difference-between-sdk-and-api/)
- GitHub issue: [#388](https://github.com/torrust/torrust-tracker-deployer/issues/388)
