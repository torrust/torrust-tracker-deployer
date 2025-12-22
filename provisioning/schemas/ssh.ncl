# SSH Credentials Schema
# Type contract for SSH authentication configuration

{
  SshCredentials = {
    # Required (usually provided by user)
    private_key_path | String | optional,
    public_key_path | String | optional,

    # Optional (have defaults)
    username | String | optional,
    port | Number | optional,
  },
}
