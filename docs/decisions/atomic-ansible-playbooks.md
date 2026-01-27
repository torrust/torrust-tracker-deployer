# Decision: Atomic Ansible Playbooks

## Status

Accepted

## Date

2026-01-27

## Context

During issue #292 a contributor added storage-creation tasks into existing playbooks (`deploy-compose-files.yml` and `deploy-grafana-provisioning.yml`). That violated the Single Responsibility Principle for playbooks and relied on Ansible `when:` gating for service enablement. The rule to keep playbooks atomic was implicit, leading to drift and repeat risk. We need an explicit decision that defines atomic playbooks, locates conditional logic in Rust (command/step orchestration), and documents registration requirements for static playbooks.

## Decision

- **Atomic playbooks**: Each playbook performs exactly one conceptual responsibility.
- **Rust-driven gating**: Service/feature enablement checks live in Rust commands/steps. Ansible `when:` must not be used to decide whether a service is enabled; it is allowed only for host-fact guards (e.g., distro, kernel capability).
- **One feature → one playbook + one step**: New behavior introduces a dedicated playbook and a Rust step that conditionally executes it.
- **Static playbook registration**: All static playbooks must be registered in `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs` under `copy_static_templates()` so they are copied to the build directory.
- **Naming**: Playbook names describe a single action (avoid "and" compounds); Tera is for templating variables only, not for control logic.

## Consequences

- ✅ Clear failure domains and simpler reviews (one responsibility per playbook).
- ✅ Better composability and testability (steps can be toggled independently from Rust).
- ✅ Reduced risk of regressions from unrelated task additions.
- ⚠️ Slight increase in playbook count; requires diligence in registering static files.

## Alternatives Considered

- Continue adding tasks to existing playbooks: rejected due to coupling and unclear responsibilities.
- Keep Ansible `when:` for service enablement: rejected to keep orchestration logic in Rust where we have richer typing and traceability.
- Use larger, multi-responsibility playbooks with internal conditionals: rejected for testability and review clarity.

## Related Decisions

- [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md)
- [External Tool Adapters Organization](./external-tool-adapters-organization.md)
- [Execution Context Wrapper Pattern](./execution-context-wrapper.md)

## References

- Issue #292 (original violation)
- Issue #306 (this decision)
- AGENTS rule #8
- Ansible guide: `docs/contributing/templates/ansible.md`
