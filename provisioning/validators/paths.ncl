# Path Validators
# Validators for file paths and configuration paths

{
  # Validates SSH key path (non-empty string)
  #
  # Note: This validator does not check if the file actually exists.
  # File existence is verified when the configuration is used by the
  # Rust application or when provisioning occurs.
  #
  # Args:
  #   path: String - path to SSH key file
  # Returns:
  #   path if valid format
  # Throws:
  #   Error if path is empty
  ValidSshKeyPath = fun path =>
    if std.string.length path > 0 then path
    else std.record.fail "SSH key path cannot be empty",

  # Validates generic file path (non-empty string)
  #
  # Args:
  #   path: String - file path to validate
  # Returns:
  #   path if valid format
  # Throws:
  #   Error if path is empty
  ValidPath = fun path =>
    if std.string.length path > 0 then path
    else std.record.fail "Path cannot be empty",
}
