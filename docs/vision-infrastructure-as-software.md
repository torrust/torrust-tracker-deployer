# Vision: The Future of Infrastructure as Code

## Where is Infrastructure Code Evolving To?

Infrastructure as Code (IaC) has matured significantly over the past decade, but many of today's tools (Terraform, Ansible, etc.) have reached their limits. They provided a huge leap forward in automation and repeatability, but their abstractions are often too coupled to specific providers, too imperative, and too hard to integrate into higher-level software systems.

The industry is beginning to move toward **Infrastructure as Software** rather than just Infrastructure as Code. This means treating infrastructure not as static configuration files, but as fully testable, observable, and composable software components. Some examples of this trend include tools like Pulumi (using general-purpose languages for infra), Crossplane (treating infra as Kubernetes objects), or CDK (AWS's Cloud Development Kit). The shift emphasizes **domain-driven design (DDD) principles** applied to infrastructure: explicit contracts, separation of concerns, and infrastructure modeled as part of the broader system design.

---

## Domain-Driven Layers for an Infrastructure Application

Borrowing from DDD, we can think of infrastructure applications as having layers:

1. **Domain Layer**

   - Expresses business-level requirements for infrastructure (e.g., "we need a resilient tracker cluster with persistent storage").
   - Independent of any specific provider (AWS, GCP, bare metal, etc.).

2. **Application Layer**

   - Orchestrates workflows and policies (e.g., scaling rules, deployment strategies, backups).
   - Integrates infrastructure operations into business workflows.

3. **Infrastructure Layer**

   - Concrete implementation details: Terraform modules, Ansible playbooks, Kubernetes manifests, etc.
   - Interfaces with cloud providers, virtualization systems, and container runtimes.

This separation would make infrastructure software easier to evolve and maintain, reducing tool lock-in and increasing alignment with the overall system architecture.

---

## Current Problems in Infrastructure Code

While IaC has solved many problems, today's tools show some common pain points:

- **Testing Gaps**: Most infra code is difficult to test locally or in CI/CD without deploying real resources.
- **Observability**: Tools are often black boxes with little insight into execution plans, state drift, or failures.
- **Tool Coupling**: Strong dependency on specific vendors (e.g., Terraform + AWS), making systems hard to port or extend.
- **Integration Issues**: Infrastructure logic is isolated from domain logic, making it hard for applications to "know" about their own infra requirements.
- **Complexity**: Declarative tools grow unmanageable at scale, with large, brittle configurations.

---

## Toward a New Generation of Infrastructure Tools

The next generation of infrastructure tooling should:

- Provide **first-class testing support** (unit and integration testing for infra).
- Be **observable** with proper logging, metrics, and debugging.
- Allow **composability** and integration with domain applications.
- Be **provider-agnostic** at the domain level, deferring provider choice to implementation layers.
- Support **evolutionary design**, so infra can adapt as the system grows.
