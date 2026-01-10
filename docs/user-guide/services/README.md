# Services Documentation

This directory contains detailed documentation for optional services that can be included in your Torrust Tracker deployments.

## Purpose

The services documentation provides comprehensive guides for each optional service, including:

- Configuration options and examples
- Enabling/disabling instructions
- Verification and testing procedures
- Troubleshooting common issues
- Architecture and deployment details

## Available Services

- **[Prometheus Monitoring](prometheus.md)** - Metrics collection and monitoring service

  - Automatic metrics scraping from tracker API endpoints
  - Web UI for querying and visualizing metrics
  - Configurable scrape intervals
  - Enabled by default, can be disabled

- **[Grafana Visualization](grafana.md)** - Metrics visualization and dashboards
  - Web-based dashboard interface for visualizing Prometheus metrics
  - Configurable admin credentials
  - Auto-import of tracker dashboards (planned)
  - Requires Prometheus to be enabled
  - Enabled by default, can be disabled

## Service Organization

Each service guide follows a consistent structure:

1. **Overview** - Purpose and capabilities
2. **Default Behavior** - Out-of-the-box configuration
3. **Configuration** - How to configure the service
4. **Disabling** - How to remove the service from deployment
5. **Accessing** - How to interact with the service after deployment
6. **Verification** - How to verify the service is working correctly
7. **Troubleshooting** - Common issues and solutions
8. **Architecture** - Technical details about deployment structure

## How Services Work

Services in the deployer are:

- **Optional** - Include only what you need
- **Configuration-based** - Enable by adding a section to your environment JSON
- **Containerized** - Each service runs in its own Docker container
- **Integrated** - Automatically configured to work with the tracker

### Adding a Service

To include a service in your deployment, add its configuration section to your environment JSON file:

```json
{
  "environment": {
    "name": "my-env"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust"
  },
  "prometheus": {
    "scrape_interval": 15
  }
}
```

### Removing a Service

To exclude a service from your deployment, simply remove its configuration section:

```json
{
  "environment": {
    "name": "my-env"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust"
  }
  // No prometheus section = service not deployed
}
```

## Future Services

As the deployer evolves, additional optional services may be added to this directory:

- Database services (MySQL, PostgreSQL)
- Reverse proxy services (Nginx, Traefik)
- Logging aggregation (Loki, Elasticsearch)
- Alerting services (Alertmanager)

## Related Documentation

- **[User Guide](../README.md)** - Main user guide with general configuration
- **[Quick Start Guides](../quick-start/README.md)** - Getting started with deployments
- **[Configuration Reference](../configuration/)** - Environment configuration details
- **[Manual Testing Guides](../../e2e-testing/manual/)** - Service verification procedures

## Contributing

When adding new service documentation:

1. Follow the established structure outlined above
2. Include practical examples and commands
3. Provide verification steps
4. Document common troubleshooting scenarios
5. Update this README to list the new service
6. Add cross-references to related documentation

See [Contributing Guidelines](../../contributing/README.md) for more details.
