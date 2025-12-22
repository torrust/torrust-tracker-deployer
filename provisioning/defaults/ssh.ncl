# SSH Defaults
# Default values with type contracts applied

let schemas = import "../schemas/ssh.ncl" in

{
  ssh_credentials | schemas.SshCredentials = {
    # private_key_path must be provided by user (no default)
    # public_key_path must be provided by user (no default)

    # Default SSH username: "torrust" (project convention)
    username = "torrust",

    # Default SSH port: (standard SSH port)
    port = 22,
  },
}
