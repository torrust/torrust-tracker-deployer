# Username Validators (Linux system username rules)
# Mirrors validation from src/shared/username.rs

{
  # Validates Username according to Linux system requirements
  #
  # Rust reference: src/shared/username.rs
  # Used for: SSH authentication, system user creation, process ownership
  #
  # Rules (MUST match Rust exactly):
  # 1. Non-empty
  # 2. 1-32 characters maximum
  # 3. Must start with letter (a-z, A-Z) or underscore (_)
  # 4. Subsequent chars: letters, digits, underscores, hyphens
  # 5. Case-sensitive (allows uppercase, unlike EnvironmentName)
  #
  # Valid examples: torrust, _service, Deploy_USER, user-123, Admin
  # Invalid examples: 123user, -user, user@domain, user.name
  #
  # Args:
  #   username: String - username to validate
  # Returns:
  #   username if valid
  # Throws:
  #   Error with specific rule violation
  ValidUsername = fun username =>
    let len = std.string.length username in

    # Rule 1: Check if empty
    let _ = if username == "" then
      std.record.fail "Username cannot be empty"
    else
      username
    in

    # Rule 2: Check length (1-32 characters)
    let _ = if len > 32 then
      std.record.fail "Username must be 32 characters or less, got %{std.string.from_number len} characters"
    else
      username
    in

    # Rule 3: Check first character (must be letter or underscore)
    let first_char = std.string.characters username |> std.array.at 0 in
    let _ = if !std.string.is_match "^[a-zA-Z_]" first_char then
      std.record.fail "Username must start with a letter (a-z, A-Z) or underscore (_)"
    else
      username
    in

    # Rule 4: Check all characters (letters, digits, underscores, hyphens only)
    let invalid_chars_list = std.string.split "" username |
      std.array.filter (fun c =>
        !std.string.is_match "^[a-zA-Z0-9_-]$" c
      ) in

    let _ = if std.array.length invalid_chars_list > 0 then
      std.record.fail "Username must contain only letters, digits, underscores, and hyphens"
    else
      username
    in

    # All validations passed
    username,
}
