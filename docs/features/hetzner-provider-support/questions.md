# Clarifying Questions for Hetzner Provider Support

This document contains questions to clarify requirements, scope, and priorities before implementation begins. Product owners or stakeholders should answer these questions directly in the document.

---

## 🔍 Scope and Requirements

### 1. **Core Functionality**

**Question**: What is the minimum viable functionality for this feature?

**Your Answer**: ✅ Confirmed - Initial thoughts are correct.

**Initial Thoughts**:

- Ability to provision a Hetzner Cloud VM using the `provision` command
- Same workflow as LXD: provision → configure → test
- User specifies provider in environment configuration

---

### 2. **Out of Scope**

**Question**: What is explicitly NOT included in this feature?

**Your Answer**: ✅ Confirmed - Initial thoughts are correct.

**Initial Thoughts**:

- Provider abstraction layer (no common interface for provider parameters)
- Other cloud providers (AWS, GCP, DigitalOcean, etc.) - future work
- Migration between providers
- Multi-provider deployments (one environment = one provider)

---

### 3. **User Experience**

**Question**: How should users interact with this feature? What's the expected workflow?

**Your Answer**:

Provider configuration is part of the environment JSON and only used during the `provision` command.

**Open Design Decision**: Single JSON file vs. two separate files?

| Approach                               | Pros                                               | Cons                                                |
| -------------------------------------- | -------------------------------------------------- | --------------------------------------------------- |
| **Single JSON**                        | Simpler to pass around; one file = one environment | Validation complexity with provider-specific fields |
| **Two JSON files** (common + provider) | Cleaner validation per file                        | More files to manage; need to associate them        |

**Main Concern**: How to validate user inputs when the schema varies by provider?

**Analysis of validation approaches for single JSON**:

1. **Two-phase validation**: First validate common fields, then validate provider-specific section based on `provider` field value. This is a common pattern (e.g., Kubernetes resources validate `apiVersion`/`kind` first, then spec).

2. **JSON Schema with `oneOf`/`anyOf`**: JSON Schema supports conditional validation - the `provider_config` field could use `oneOf` to select between LXD and Hetzner schemas.

3. **Rust enum deserialization**: Serde's `#[serde(tag = "provider")]` or adjacently tagged enums handle this naturally:

   ```rust
   #[serde(tag = "provider")]
   enum ProviderConfig {
       #[serde(rename = "lxd")]
       Lxd,  // No extra fields needed
       #[serde(rename = "hetzner")]
       Hetzner(HetznerConfig),
   }
   ```

**Recommendation**: Single JSON file with provider-specific section. Rust's serde handles polymorphic deserialization well, and it keeps the "one environment = one file" mental model.

**Initial Thoughts**:

- User creates environment JSON with `provider: "hetzner"` field
- User provides Hetzner-specific parameters (API token, server type, location, etc.)
- Commands work the same: `provision`, `configure`, `test`, `destroy`

---

## 🎯 Technical Approach

### 4. **Provider Selection Mechanism**

**Question**: How should the user specify which provider to use?

Options:

- A) In environment JSON file (recommended for flexibility)
- B) Via command line argument
- C) Via environment variable
- D) Separate command per provider (e.g., `provision-lxd`, `provision-hetzner`)

**Your Answer**: ✅ Option A - In environment JSON file.

---

### 5. **Provider-Specific Configuration**

**Question**: Should we require provider-specific configuration in the same format as the native tools use, or create our own simplified format?

Example: Hetzner server types are `cx11`, `cx21`, `cpx11`, etc. Do we:

- A) Use Hetzner's native naming directly
- B) Create our own abstraction (e.g., `small`, `medium`, `large`)

**Your Answer**: ✅ Option A - Use native naming directly. No abstraction layer - it would require significant effort to map provider parameters into our own abstraction, and adds maintenance burden when providers change their offerings.

---

### 6. **LXD Provider Naming**

**Question**: What should we call the LXD provider now that we're making it explicit?

Options:

- A) `lxd` - Simple and direct
- B) `local` - Emphasizes it's for local development
- C) `lxd-vm` - Clarifies we use VMs, not containers

**Your Answer**: ✅ Option A - `lxd`. We officially only support VMs; containers are only used internally for testing.

---

### 7. **Default Provider**

**Question**: Should there be a default provider if none is specified?

Options:

- A) No default - always require explicit selection (clearer, but more verbose)
- B) Default to `lxd` for backward compatibility
- C) Default based on context (e.g., if Hetzner token is present, use Hetzner)

**Your Answer**: ✅ Option A - No default. Always require explicit provider selection. This is clearer and keeps flexibility to use different providers for testing in the future.

---

### 8. **API Token Handling**

**Question**: How should sensitive credentials (like Hetzner API token) be handled?

Options:

- A) Environment variable only (`HCLOUD_TOKEN` - matches Hetzner CLI)
- B) In environment JSON (encrypted or plaintext)
- C) Both options supported

**Your Answer**: ✅ Option B - In environment JSON (plaintext).

**Rationale**: This is a single-use deployment tool, not a management platform or sysadmin replacement. The entire local configuration (`data/`, `envs/`, `build/` directories) is considered sensitive and should not be shared. Users are responsible for securing their local deployment data after using the tool. Storing the token in the environment JSON keeps all configuration in one place and simplifies the single-use workflow.

---

### 9. **Terraform State Management**

**Question**: Should Terraform state be stored locally or remotely for Hetzner deployments?

Options:

- A) Local only (simpler, matches current LXD approach)
- B) Remote state backend (S3, Terraform Cloud, etc.) for team collaboration
- C) Local by default, remote as optional configuration

**Your Answer**: ✅ Option A - Local only.

**Rationale**: This is not a multi-user or team deployment management tool. The `destroy` command exists primarily to allow users to undo and start from scratch if they make a configuration mistake or want to change something - not as a routine infrastructure management operation. Remote state adds unnecessary complexity for a single-use deployment tool.

---

## 📊 Priority and Timeline

### 10. **Priority Level**

**Question**: What is the priority of this feature? (High | Medium | Low)

**Your Answer**: ✅ **High**. Without this feature the deployer is essentially useless - it cannot deploy to production infrastructure.

---

### 11. **Timeline Expectations**

**Question**: Is there a target date or sprint for completion?

**Your Answer**: Target is a fully functional deployer before the end of 2025. This is one of several pending tasks in the roadmap. However, **quality is not negotiable** - we will postpone if necessary to ensure a good quality deployer rather than rushing to meet the deadline.

---

### 12. **Implementation Phases**

**Question**: Should we implement both phases (LXD refactor + Hetzner) together, or release Phase 1 separately?

Options:

- A) Release Phase 1 (LXD explicit) first, then Phase 2 (Hetzner)
- B) Implement both phases together, release as one feature
- C) Implement incrementally with multiple releases

**Your Answer**: ✅ Option C - Implement incrementally. Refactor first whatever is needed to make it easy to add the second provider, then add Hetzner. This allows validating the architecture at each step and keeps changes focused and reviewable.

---

## ✅ Success Criteria

### 13. **Definition of Done - Phase 1**

**Question**: How do we know Phase 1 (LXD explicit provider) is complete?

**Your Answer**: ✅ Confirmed - Suggested criteria are correct.

**Criteria**:

- [ ] Environment JSON requires explicit `provider: "lxd"` field
- [ ] Existing tests pass with updated configuration
- [ ] Documentation updated to show provider selection
- [ ] Clear error message if provider is missing from config

---

### 14. **Definition of Done - Phase 2**

**Question**: How do we know Phase 2 (Hetzner support) is complete?

**Your Answer**: ✅ Confirmed - Suggested criteria are correct.

**Criteria**:

- [ ] Can provision Hetzner VM using `provision` command
- [ ] Can configure Hetzner VM (Docker, security)
- [ ] Can destroy Hetzner infrastructure
- [ ] E2E test for Hetzner workflow (if feasible without real cloud costs)
- [ ] Documentation for Hetzner setup

---

### 15. **Testing Requirements**

**Question**: How should we test Hetzner provider without incurring cloud costs in CI?

Options:

- A) Mock tests only
- B) Real Hetzner tests gated behind environment variable
- C) Use Hetzner's test/trial account
- D) Manual testing only for Hetzner, automated for LXD

**Your Answer**: ✅ Option D - Manual testing for Hetzner, automated for LXD.

**Manual Testing**: The product owner will provide the Hetzner API token for manual E2E testing during development. This ensures the feature works as expected on real infrastructure before release.

**Automated Testing (CI)**: For now, automated tests use LXD only.

**Future Plan**: Add a dedicated GitHub Actions workflow per provider that:

1. Is manually triggered (workflow_dispatch) with token from GitHub Secrets
2. Runs the deployer to create a new environment on Hetzner
3. Runs a full E2E test against the provisioned infrastructure
4. Destroys the Hetzner environment

This allows real cloud testing on-demand without incurring continuous CI costs.

---

## ⚠️ Risk Assessment

### 16. **Backward Compatibility**

**Question**: How should we handle existing environment JSON files that don't have a provider field?

Options:

- A) Error - require explicit provider (breaking change, but clear)
- B) Default to `lxd` for backward compatibility
- C) Migration script to update existing files

**Your Answer**: ✅ Option A - Error with clear message. Backward compatibility is not a concern.

**Rationale**: The deployer is not in production use yet - there are no external users. We are free to make breaking changes without migration paths. This simplifies the implementation and keeps the codebase clean.

---

### 17. **Hetzner-Specific Risks**

**Question**: Are there Hetzner-specific considerations we should be aware of?

Topics:

- Hetzner API rate limits
- Hetzner regions/availability
- IPv4 vs IPv6 considerations
- Firewall configuration (Hetzner Cloud Firewall vs OS firewall)

**Your Answer**: ✅ No significant Hetzner-specific risks identified.

**Firewall Decision**: We use OS-level firewall (UFW) instead of Hetzner Cloud Firewall. This makes the deployment portable - if users need to migrate to another provider, the firewall configuration moves with the VM.

---

### 18. **Cost Management**

**Question**: Should we include cost estimation or warnings in the provisioning output?

**Your Answer**: ✅ No. Users are responsible for researching costs themselves.

**Rationale**: Cloud provider pricing changes frequently. Maintaining cost information would add significant maintenance burden for little value. Users should consult Hetzner's pricing page directly before provisioning.

---

## 💡 Additional Questions

### 19. **Domain Concepts**

**Question**: The domain currently has `InstanceName` and `ProfileName` which are LXD-specific. Should we:

Options:

- A) Rename to generic names (e.g., `ResourceId`)
- B) Keep LXD names and add Hetzner equivalents
- C) Make them provider-agnostic but keep current names

**Your Answer**: ✅ Mixed approach:

- **`InstanceName`**: Keep as a generic domain concept. All providers need an instance/server name. We use the most restrictive validation rules that work across all providers (currently: RFC 1123 minus periods - see specification.md for details).

- **`ProfileName`**: Move to LXD-specific configuration. Other providers like Hetzner don't have an equivalent concept.

---

### 20. **Environment JSON Schema**

**Question**: Should we version the environment JSON schema to handle provider additions?

**Your Answer**: ✅ Yes. Version the schema to handle future changes and provider additions gracefully.

---

### 21. **Provider-Specific Features**

**Question**: Should we expose provider-specific features (e.g., Hetzner snapshots, volumes)?

Options:

- A) Not initially - keep to basic VM provisioning
- B) Yes, as optional configuration
- C) Defer until after basic implementation

**Your Answer**: ✅ Option A - Not initially. Keep to basic VM provisioning.

**Rationale**: Focus on the core functionality first. Provider-specific features like snapshots and volumes can be added later if there's demand.

---

## 📝 Notes

Use this section for any additional context, clarifications, or decisions made during the Q&A process.

### Analysis Summary

Based on the codebase analysis (see [analysis.md](./analysis.md)), the key integration points are:

1. **Environment Configuration** (`UserInputs`): Add provider selection field
2. **OpenTofu Templates**: Create `templates/tofu/hetzner/` directory
3. **Template Renderer**: Update `TofuTemplateRenderer` to select provider templates
4. **Provision Handler**: No changes needed if template renderer is updated correctly
5. **Domain Types**: `InstanceName` and `ProfileName` work for both providers

### Recommended Approach

The "refactor first" strategy is confirmed as the right approach:

1. **Phase 1**: Make LXD explicit without adding new code
2. **Phase 2**: Add Hetzner with minimal changes to existing architecture
