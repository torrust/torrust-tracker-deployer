# Environment Defaults
# Default values with type contracts applied

let schemas = import "../schemas/environment.ncl" in

{
  environment | schemas.Environment = {
    # name is required from user (no default)
    # instance_name is optional
  },
}
