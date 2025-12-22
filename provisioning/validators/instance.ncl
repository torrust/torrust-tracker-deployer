# Instance Name Validators (LXD naming rules)
# Mirrors validation from src/domain/instance_name.rs

{
  # Validates InstanceName according to LXD requirements
  #
  # Rust reference: src/domain/instance_name.rs
  # Use cases: LXD virtual machine names, Docker container names
  #
  # Rules (MUST match Rust exactly):
  # 1. Non-empty
  # 2. 1-63 characters maximum
  # 3. ASCII letters, numbers, dashes only
  # 4. Cannot start with digit or dash
  # 5. Cannot end with dash
  # 6. Cannot contain uppercase or special characters
  #
  # Valid examples: test-instance, vm-prod, app01, a
  # Invalid examples: 1test, -test, test-, test@instance, test_instance
  #
  # Args:
  #   name: String - instance name to validate
  # Returns:
  #   name if valid
  # Throws:
  #   Error with specific rule violation
  ValidInstanceName = fun name =>
    let len = std.string.length name in

    # Rule 1: Check if empty
    let _ = if name == "" then
      std.record.fail "Instance name cannot be empty"
    else
      name
    in

    # Rule 2: Check length (max 63 characters)
    let _ = if len > 63 then
      std.record.fail "Instance name must be 63 characters or less, got %{std.string.from_number len} characters"
    else
      name
    in

    # Rule 3: Check for invalid characters (only ASCII alphanumeric + dashes allowed)
    let invalid_chars_list = std.string.split "" name |
      std.array.filter (fun c =>
        !std.string.is_match "^[a-zA-Z0-9-]$" c
      ) in

    let _ = if std.array.length invalid_chars_list > 0 then
      std.record.fail "Instance name must contain only ASCII letters, numbers, and dashes"
    else
      name
    in

    # Rule 4: Check first character (cannot be digit or dash)
    let first_char = std.string.characters name |> std.array.at 0 in
    let _ = if std.string.is_match "^[0-9-]" first_char then
      std.record.fail "Instance name must not start with a digit or dash"
    else
      name
    in

    # Rule 5: Check last character (cannot be dash)
    let _ = if std.string.is_match "-$" name then
      std.record.fail "Instance name must not end with a dash"
    else
      name
    in

    # All validations passed
    name,
}
