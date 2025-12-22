# Provider Defaults
# No defaults provided (user must supply provider configuration)

let schemas = import "../schemas/provider.ncl" in

{
  # provider field must be completely supplied by user
  # Example (user would provide):
  # provider | schemas.Provider = {
  #   provider = "lxd",
  #   profile_name = "my-profile",
  # }
}
