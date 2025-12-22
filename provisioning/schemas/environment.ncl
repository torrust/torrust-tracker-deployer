# Environment Schema
# Type contract for environment identification

{
  Environment = {
    name | String | optional,              # Provided by user (no default)
    instance_name | String | optional,     # Optional, auto-generated if not provided
  },
}
