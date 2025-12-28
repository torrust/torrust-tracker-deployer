# Provisioning Documentation

Additional documentation for the Torrust Tracker provisioning system.

## 📚 Available Documents

### [quickstart.md](quickstart.md)

Quick reference guide for getting started with environment provisioning.

**Contents:**

- Prerequisites and installation
- Basic workflow (configure.sh/configure.nu → Nickel → JSON → Deployment)
- Three configuration options (nickel-roundtrip wizard, manual JSON, advanced Nickel)
- Quick commands reference
- Common troubleshooting

**Use when:** You need to quickly set up your first environment without deep understanding.

---

### [nickel-roundtrip.md](nickel-roundtrip.md)

Technical documentation for the TypeDialog ↔ Nickel roundtrip integration.

**Contents:**

- Workflow architecture diagram (configure.sh/configure.nu)
- Template system (config-template.ncl.j2)
- Constraint synchronization (constraints.toml → form + validators)
- Validation layers (TypeDialog → Nickel → Rust)
- Multi-backend support (cli, tui, web)
- Testing and verification procedures

**Use when:** You need to understand or modify the nickel-roundtrip workflow or the configure.sh/configure.nu scripts.

---

## 🔗 Related Documentation

### Main Documentation

- **[../README.md](../README.md)** - Complete provisioning system guide
- **[../CHANGELOG.md](../CHANGELOG.md)** - Change history and version updates

### Subdirectory Documentation

- **[../constraints/README.md](../constraints/README.md)** - Validation constraints (single source of truth)
- **[../schemas/README.md](../schemas/README.md)** - Nickel type contracts
- **[../defaults/README.md](../defaults/README.md)** - Default configuration values
- **[../validators/README.md](../validators/README.md)** - Nickel validation functions
- **[../values/README.md](../values/README.md)** - User configuration examples
- **[../fragments/README.md](../fragments/README.md)** - TypeDialog form fragments
- **[../templates/README.md](../templates/README.md)** - Nickel template documentation

### Project Documentation

- **[../../docs/decisions/](../../docs/decisions/)** - Architectural Decision Records (ADRs)
- **[../../docs/technical/](../../docs/technical/)** - Technical implementation guides
- **[../../docs/user-guide/](../../docs/user-guide/)** - End-user guides

---

## 📖 Documentation Navigation

```text
provisioning/
├── README.md                    # Main provisioning guide
├── CHANGELOG.md                 # Version history
├── docs/                        # Additional documentation (this directory)
│   ├── README.md               # This file
│   ├── quickstart.md           # Quick start guide
│   └── nickel-roundtrip.md     # Technical roundtrip documentation
├── constraints/README.md        # Constraint definitions
├── schemas/README.md            # Type contracts
├── defaults/README.md           # Default values
├── validators/README.md         # Validation logic
├── values/README.md             # User configurations
├── fragments/README.md          # Form fragments
└── templates/README.md          # Template documentation
```

---

**Last updated:** December 28, 2025
