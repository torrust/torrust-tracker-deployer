# Roadmap Questions and Clarifications

This document contains questions about the [Torrust Tracker Deployer Roadmap](./roadmap.md) to help clarify scope, priorities, and implementation det**Q9.1:** Scalability considerations:

- What's the maximum expected environment size? **Very small.**
- How do we handle resource limits and quotas? **Irrelevant for now.**
- Performance implications of the incremental deployment approach? **It's not a big deal because we are not going to have big environments. The deployer is not designed to handle big environments with many instances. It's designed to handle small environments with one instance.**

## Scope and Requirements Questions

### 1. User Experience and Interface Design

**Q1.1:** What is the target user persona for this tool? Are we targeting:

- DevOps engineers familiar with infrastructure tools?
- Developers who want simple deployment without infrastructure knowledge?
- System administrators managing production deployments?

**Answer:** Mainly developers who want simple deployment without infrastructure knowledge. However, system administrators managing production deployments could also benefit from it, especially if they are comfortable with the way the deployer installs and configures services.

**Q1.2:** For the console commands (create, destroy, deploy, status, test), what level of interactivity is expected?

- Should these be fully automated with config files?
- Do we need interactive prompts for missing configuration?
- Should there be confirmation steps for destructive operations?

**Answer:** In the first version, we want these to be fully automated with config files. Commands should stop when there are missing configuration values and report a clear error. The only exception is the `destroy` command, which should ask for confirmation before proceeding.

**Q1.3:** What's the expected learning curve? Should the tool be:

- Self-documenting with extensive help and examples?
- Assume familiarity with Docker, Ansible, and infrastructure concepts?

**Answer:** It should be self-documenting with extensive help and examples. However, for experienced users, it should allow access to low-level details if they want them.

### 2. Configuration and Environment Management

**Q2.1:** How will users provide configuration values for the `create` command?

- Configuration files (YAML, TOML, JSON)?
- Command-line arguments?
- Interactive prompts?
- Environment variables?

**Answer:** The initial plan is to use configuration files (TOML) and environment variables because we have nested configuration structures (like arrays of trackers in the tracker TOML configuration). This approach is also aligned with other tools like the Torrust Tracker itself.

**Q2.2:** What's the scope of environment customization?

- Pre-defined templates with limited customization?
- Full flexibility to customize all infrastructure aspects?
- Preset configurations for different use cases (dev, staging, production)?

**Answer:** Users will only provide configuration values for predefined templates. Basically, the user has to provide the values that make an environment unique (like domain name, instance size, etc.) and the deployer will handle the rest using the templates.

**Q2.3:** Should environments support:

- Multi-region deployments? **No.**
- Different instance sizes/types? **Yes, when we implement the Hetzner provider.**
- Custom networking configurations? **Only firewall rules for now.**

## Execution Order and Dependencies

### 3. Implementation Sequence

**Q3.1:** Why is point 1 (main app scaffolding) prioritized over infrastructure providers?

- Is LXD sufficient for early development and testing?
- Should Hetzner provider development start earlier to validate the abstraction?

**Answer:** We expect Hetzner to be easy to implement - it will only need new OpenTofu templates. The main complexity will be that the user's input will have two different parts: one for the common configuration and another for the provider-specific configuration. But this can be handled when we implement the Hetzner provider.

On the other hand, we have already implemented the Hetzner provider in previous proof of concepts, so we know it will be straightforward to implement.

**Q3.2:** For point 3 (application commands), should ConfigureCommand be fully completed before starting ReleaseCommand/RunCommand?

- Are there dependencies between firewall configuration and service deployment?
- Can these be developed in parallel?

**Answer:** ConfigureCommand should be finished before starting ReleaseCommand/RunCommand. There are no dependencies between them. The firewall configuration is a basic configuration to close all ports except SSH. Later, when we add more services, we will need to update the firewall configuration to open the required ports for each service.

**Q3.3:** The Docker image (point 4) seems like it could enable easier testing of points 1-3. Should it be prioritized earlier?

**Answer:** We already use virtual machines and Docker for testing. Point 4 would be the full public official Docker image for the deployer.

### 4. Service Deployment Strategy

**Q4.1:** For the incremental service approach in 3.2, what's the deployment validation strategy?

- How do we ensure each service addition doesn't break existing services?
- Should there be automated health checks between each service addition?

**Answer:** Each Ansible playbook should have its own validations for preconditions and postconditions. Besides that, we can incrementally add more functionality to the TestCommand depending on the environment state and the services that are deployed.

**Q4.2:** What's the rollback strategy if a service addition fails?

- Should each slice be independently deployable and removable?
- How do we handle service dependencies (e.g., Tracker depends on MySQL)?

**Answer:** Each slice is not going to be added at runtime. The slice is only a concept to incrementally add more services to the deployer. After implementing the first slice, users will be able to deploy a hello-world Docker container instead of the real tracker. After the second iteration, they will be able to deploy a stack with MySQL. After the third iteration, they will be able to deploy the tracker connected to MySQL, and so on. At this point, the tracker will be operational but without the extra monitoring services like Prometheus and Grafana.

Note that after each step we only deploy a fully working Docker Compose stack. We don't have to handle rollbacks or partial deployments.

On the other hand, we are not planning to handle rollbacks for the full deployment (the deploy command). If something fails, the user will have to fix the issue and run the deploy command again. The deploy command should be idempotent and able to resume from a failed state. If it's not possible, the user will have to destroy the environment and create it again.

## Critical Missing Aspects

### 5. Security and Production Readiness

**Q5.1:** Security considerations seem minimal in the roadmap. What about:

- SSL/TLS certificate management (Let's Encrypt integration)?
- Secrets management (API keys, database passwords)?
- Network security beyond basic firewall configuration?
- User authentication and access control?

**Answer:** This will be expanded in future iterations. The current plan is to use the Figment crate to load user inputs and save them securely. For secrets, the user can decide if they are provided in the configuration file or as environment variables (same approach as the Torrust Tracker).

**Action:** Add a point to the roadmap to add HTTPS support for the HTTP tracker, tracker API, and Grafana after having the full application running on HTTP.

**Q5.2:** Backup and disaster recovery:

- Database backups for MySQL?
- Configuration backups?
- Recovery procedures documentation?

**Answer:** This was included in previous proof of concepts, so we know how to implement it.

**Action:** Add a point to the roadmap for backup and disaster recovery in future iterations.

**Q5.3:** Monitoring and observability:

- Log aggregation and management?
- Alerting integration beyond Prometheus/Grafana?
- Performance monitoring and tuning?

### 6. Testing and Quality Assurance

**Q6.1:** What's the testing strategy for the incremental deployments?

- Integration tests for each service slice?
- End-to-end testing of the complete stack?
- Performance testing under load?

**Answer:** Only E2E tests for provisioning (already implemented) and E2E tests for the rest of the deployment process (simulate provisioned VM with Docker). This is also already implemented. We just need to expand the tests incrementally as we add more services.

**Q6.2:** How do we ensure consistency across different infrastructure providers?

- Cross-provider testing strategy?
- Provider-specific feature parity validation?

Manual testing for now. Automated tests for non-local providers can be expensive.

### 7. Documentation and User Support

**Q7.1:** What documentation is planned beyond the technical docs?

- User guides for different deployment scenarios?
- Troubleshooting guides for common issues?
- Migration guides for existing setups?

Only basic user guides and troubleshooting guides for now. We should write them as we implement the features.

**Q7.2:** Error handling and user guidance:

- How detailed should error messages be? **As detailed as possible depending on the verbosity level.**
- Should there be automated diagnostic tools? **Not many, there will be commands to check if the third-party tools are installed and working correctly.**
- Recovery suggestions for common failure scenarios? **Yes, definitely.**

## Risk Assessment

### 8. Technical Risks

**Q8.1:** Provider abstraction complexity:

- How do we handle provider-specific features that don't map well to abstractions?
- What's the fallback if a provider's API changes significantly?

**Answer:** We won't have abstractions per provider. Each provider will have its own configuration structure and the user will have to provide the configuration values for the selected provider. The common configuration values will be shared between all providers.

There is no fallback if a provider's API changes significantly. We will have to update the deployer to handle the new API. This is a risk that we have to accept when using third-party services.

**Q8.2:** Service orchestration complexity:

- How do we handle service startup order and dependencies? **We have to handle that in the best way possible. For example, we have a step to wait for cloud-init to finish before starting to configure the services. We can also add retries with exponential backoff when connecting to services that could not be ready yet (like MySQL).**
- What happens when services fail to start or become unhealthy? **We only show and log the error and stop the deployment. The user will have to fix the issue and run the deploy command again.**

**Q8.3:** State management:

- How do we handle partial deployments or interrupted processes? **The environment state is already tracked and persisted, so we can resume from the last known good state.**
- What's the strategy for state reconciliation after failures? **Failed states are not recoverable automatically. If the state is corrupted, the user will have to destroy the environment and create it again.**

### 9. Operational Risks

**Q9.1:** Scalability considerations:

- What's the maximum expected environment size? Very small.
- How do we handle resource limits and quotas? Irrelevant for now.
- Performance implications of the incremental deployment approach? It's not a big deal because we are not going to have big environments. The deployer is not designed to handle big environments with many instances. It's designed to handle small environments with one instance.

**Q9.2:** Maintenance burden:

- How do we handle updates to underlying services (MySQL, Tracker, etc.)? **We don't. The maintenance is out-of-scope for now. That's why we use Docker containers for the services, so the user can update them independently if needed.**
- Dependency management for the tool itself? **Very simple for now. We will use Cargo to manage the dependencies. There are some scripts to install the required third-party tools (OpenTofu, Ansible, etc.).**
- Long-term support and backward compatibility? **Not planned for now. It will depend on user feedback and the adoption of the tool.**

### 10. Project Management Risks

**Q10.1:** Resource allocation:

- What's the expected team size and expertise? **1 developer with experience in Rust.**
- Are there external dependencies on other teams or projects? **On the Tracker project for new features, bug fixes, changes in configuration, but not critical.**

**Q10.2:** Timeline and scope creep:

- What's the minimum viable product (MVP) scope? **A basic implementation of the deployer with support for a single provider (Hetzner). Certificate management may be added later.**
- Which features are must-have vs. nice-to-have? **Must-have: basic deployment functionality, support for Hetzner provider. Nice-to-have: support for additional providers, certificate management.**
- How do we handle feature requests during development? **Include them in future iterations if they are not critical for the MVP.**

## Architecture and Design Questions

### 11. System Architecture

**Q11.1:** Command vs. Service boundary:

- How is the distinction between "console commands" and "internal app layer commands" maintained?
- Should there be a clear service layer that both console and internal commands use?

**Answer:** There is a clear separation. We are following DDD layering principles. The console commands are in the presentation layer and the internal app layer commands are in the application layer. The application layer uses the domain layer to implement the business logic.

We might need to rename the application command to command handler or something similar to avoid confusion, and use the "command" term for the DTOs that represent the user input for each command.

**Q11.2:** State persistence:

- How is environment state tracked and persisted? **In a JSON file in the data directory for each environment.**
- What happens to state when the tool is updated? **They should be migrated if there are breaking changes. But there are no real users yet. Not even a Beta version has been released.**
- Should state be portable between different installations of the tool? **Not planned for now.**

**Q11.3:** Concurrency and parallel execution:

- Can multiple environments be managed simultaneously? **Yes, the tool is designed to handle multiple environments concurrently, but it's not a likely use case. It might happen for testing purposes.**
- How do we handle concurrent operations on the same environment? **For now, there are simple locking mechanisms to prevent concurrent operations while saving the same environment. We could expand this in the future if needed. However, the main purpose of the tool is to handle one environment at a time.**

---

## Next Steps

✅ **Completed:** All questions have been answered and clarified.

**Actions taken based on the responses:**

1. ✅ **Updated the roadmap** with clearer scope and requirements
2. ✅ **Added missing critical aspects:**
   - Section 6: HTTPS support for HTTP tracker, tracker API, and Grafana
   - Section 7: Backup and disaster recovery capabilities
3. ✅ **Clarified execution order** - confirmed current prioritization is appropriate
4. ✅ **Documented risk mitigation strategies** through the detailed Q&A responses

**Key insights from the Q&A:**

- **Target users:** Primarily developers wanting simple deployment without infrastructure knowledge
- **Configuration approach:** TOML files + environment variables (aligned with Torrust Tracker)
- **Deployment strategy:** Incremental service slicing rather than deployment stage slicing
- **MVP scope:** Basic deployer with Hetzner provider support, HTTPS may come later
- **Architecture:** Clear DDD layering with separation between console and application commands
- **Testing:** Focus on E2E tests, expanding incrementally with new services

**Remaining tasks:**

- Implement the roadmap features as prioritized
- Create feature documentation in `docs/features/` before starting each major feature
- Update this document if new questions arise during implementation
