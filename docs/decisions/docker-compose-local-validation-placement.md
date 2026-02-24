# Decision: Docker Compose Local Validation Placement in the Infrastructure Layer

## Status

Accepted

## Date

2026-02-24

## Context

After rendering the `docker-compose.yml.tera` Tera template, the resulting
`docker-compose.yml` file may contain structural errors that Docker Compose will
reject at `run` time with a non-descriptive failure deep in the deployment
workflow. Issue [#382](../issues/382-docker-compose-template-empty-networks-key.md)
was a concrete example: an empty `networks:` key rendered when no optional
services were configured, which caused `docker compose up` to fail with
`services.tracker.networks must be a list` on the remote VM — far from the
point of origin.

To catch such errors early and provide actionable feedback, a validation step was
added that runs `docker compose config --quiet` against the rendered output
directory immediately after template rendering, before any files are uploaded to
the VM.

### The Question

The rendering pipeline has multiple candidate locations for the validation call:

```text
RenderDockerComposeTemplatesStep::execute()            ← application/steps layer
  └─ DockerComposeTemplateRenderingService::render()   ← application service layer
       └─ DockerComposeProjectGenerator::render()      ← infrastructure layer  ← chosen
            ├─ EnvRenderer::render()                   ← writes .env
            └─ DockerComposeRenderer::render()         ← writes docker-compose.yml
```

Alternatives explicitly considered:

1. **`DockerComposeRenderer::render()`** — the lowest-level renderer, inside
   infrastructure, writes only `docker-compose.yml`
2. **`DockerComposeProjectGenerator::render()`** — the infrastructure-layer
   generator, writes both `docker-compose.yml` and `.env` into the build
   directory
3. **`DockerComposeTemplateRenderingService::render()`** — the application-layer
   service that orchestrates the generator
4. **`RenderDockerComposeTemplatesStep::execute()`** — the application-layer step
   invoked by the command handler

### Constraints

1. **DDD layering**: Infrastructure operations (spawning a child process, invoking a
   CLI tool) must not leak upward into the application layer. The application layer
   should remain free of concrete system-call dependencies.
2. **Complete artifact**: `docker compose config` reads variable substitutions from
   the `.env` file in the same directory. Validating before `.env` exists produces
   false failures or incomplete results.
3. **Single responsibility alignment**: `DockerComposeRenderer::render()` is
   responsible for template rendering only; embedding a shell-command invocation
   there mixes two distinct concerns inside an already-focused collaborator.

## Decision

Call `validate_docker_compose_file()` inside
**`DockerComposeProjectGenerator::render()`**, immediately after both
`EnvRenderer` and `DockerComposeRenderer` have written their output files.

```rust
// project_generator.rs — after both render() calls succeed:
validate_docker_compose_file(&build_compose_dir)
    .map_err(|source| DockerComposeProjectGeneratorError::DockerComposeValidationFailed { source })?;
```

This is the earliest point in the pipeline where:

- the **complete artifact** (both `docker-compose.yml` and `.env`) is available,
  making the validation meaningful and accurate, and
- the code is **still inside the infrastructure layer**, respecting the DDD
  boundary that keeps process-spawning concerns out of the application layer.

### Why not `DockerComposeRenderer::render()`?

`DockerComposeRenderer` writes only `docker-compose.yml`. At that point, `.env`
does not exist yet. `docker compose config` depends on `.env` for variable
substitution, so validation would be incomplete or outright wrong for
configurations that use `.env` values.

Validating inside a renderer whose responsibility is purely template-to-file
conversion also violates single responsibility.

### Why not the application service or step?

`DockerComposeTemplateRenderingService` and
`RenderDockerComposeTemplatesStep` live in the application layer. Embedding a
call to `validate_docker_compose_file()` — which spawns a `docker` child process
— there would:

- introduce an OS-level, infrastructure dependency into the application layer,
  violating DDD layering rules,
- make the application layer harder to test in isolation without a real `docker`
  binary present.

The application layer should orchestrate business flows; it should not know how
to invoke `docker compose`.

## Consequences

### Positive

- Validation runs at the earliest correct moment: once and only once, as soon as
  the full artifact exists.
- The infrastructure layer encapsulates all `docker` CLI concerns; the application
  layer remains free of process-spawning logic.
- `DockerComposeProjectGeneratorError` gains a typed `DockerComposeValidationFailed`
  variant, surfacing the exact `docker compose config` output as an actionable
  error message to the user at `configure` time.
- Future template changes that introduce structural errors are caught immediately
  during development without needing a full deployment cycle.

### Negative / Trade-offs

- `DockerComposeProjectGenerator::render()` is now responsible for more than
  pure rendering — it also validates. This is a deliberate widening of scope:
  the generator is the "assembly" step that produces the complete artifact, and
  post-assembly validation is a natural responsibility for the assembler.
- Validation requires `docker` to be installed on the machine running the
  deployer. This is already a project dependency (Docker is listed as a required
  tool), so it adds no new requirement but does make the dependency explicit in
  the infrastructure code.

## Alternatives Considered

| Location                                          | Rejected Reason                                                          |
| ------------------------------------------------- | ------------------------------------------------------------------------ |
| `DockerComposeRenderer::render()`                 | Artifact incomplete (no `.env`); single-responsibility violation         |
| `DockerComposeTemplateRenderingService::render()` | Application layer calling infrastructure (process-spawning) violates DDD |
| `RenderDockerComposeTemplatesStep::execute()`     | Same DDD violation; too high up the call stack for an infra concern      |

## Related Decisions

- [Tera Minimal Templating Strategy](./tera-minimal-templating-strategy.md) —
  philosophy behind how templates are structured and what they are responsible for
- [Actionable Error Messages](./actionable-error-messages.md) —
  pattern used by `DockerComposeLocalValidationError::help()` to surface
  clear, solution-oriented errors

## References

- Issue [#382](../issues/382-docker-compose-template-empty-networks-key.md) —
  the concrete bug that motivated this validation step
- `src/infrastructure/templating/docker_compose/local_validator.rs` —
  implementation of `validate_docker_compose_file()`
- `src/infrastructure/templating/docker_compose/template/renderer/project_generator.rs` —
  call site within `DockerComposeProjectGenerator::render()`
