# Provider Schema
# Type contracts for infrastructure provider configuration
# Supports LXD (local) or Hetzner Cloud (managed) providers
# Note: No defaults for provider - user must supply completely

{
  # Provider record: LXD and Hetzner fields (all optional - user chooses one)
  Provider = {
    provider | String | optional,          # Provided by user
    # LXD fields
    profile_name | String | optional,
    # Hetzner fields
    api_token | doc "Hetzner fields"
    | String | optional,
    server_type | String | optional,
    location | String | optional,
    image | String | optional,
  },
}
