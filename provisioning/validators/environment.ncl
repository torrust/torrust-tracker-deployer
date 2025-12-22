# Environment Name Validators
# Mirrors validation from src/domain/environment/name.rs

let helpers = import "common.ncl" in

{
  # Validates EnvironmentName according to Rust domain rules
  #
  # Rust reference: src/domain/environment/name.rs
  #
  # Rules (MUST match Rust exactly):
  # 1. Non-empty
  # 2. Lowercase only (a-z, 0-9, -)
  # 3. Cannot start with digit
  # 4. Cannot start with dash
  # 5. Cannot end with dash
  # 6. No consecutive dashes
  # 7. Cannot contain uppercase letters
  #
  # Valid examples: dev, staging, production, e2e-config, test-integration
  # Invalid examples: Dev, 1dev, -dev, dev-, dev--test
  #
  # Args:
  #   name: String - environment name to validate
  # Returns:
  #   name if valid
  # Throws:
  #   Error with specific rule violation
  ValidEnvironmentName = fun name =>
    # Rule 1: Check if empty
    let _ = if name == "" then
      std.record.fail "Environment name cannot be empty.

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # Rule 3: Check if starts with digit
    let _ = if std.string.is_match "^[0-9]" name then
      std.record.fail "Environment name '%{name}' is invalid: starts with a number (for InstanceName compatibility).

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # Rule 7: Check for uppercase letters
    let has_uppercase = std.string.is_match "[A-Z]" name in
    let _ = if has_uppercase then
      let uppercase_chars = std.string.split "" name |
        std.array.filter (fun c => std.string.is_match "^[A-Z]$" c) |
        std.string.join "" in
      std.record.fail "Environment name '%{name}' is invalid: contains uppercase letters: %{uppercase_chars}.

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # Collect invalid characters (not in a-z, 0-9, -)
    let invalid_chars_list = std.string.split "" name |
      std.array.filter (fun c =>
        !std.string.is_match "^[a-z0-9-]$" c
      ) in

    # Rule 4 & other chars: Check for invalid characters
    let _ = if std.array.length invalid_chars_list > 0 then
      let unique_invalid = (
        invalid_chars_list |
        std.array.sort |
        std.array.reverse |
        std.array.fold (fun acc c =>
          if std.array.length acc > 0 && std.array.at (-1) acc == c then
            acc
          else
            acc @ [c]
        ) []
      ) |
      std.string.join "" in
      std.record.fail "Environment name '%{name}' is invalid: contains invalid characters: %{unique_invalid}.

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # Rule 4: Check for leading dash
    let _ = if std.string.is_match "^-" name then
      std.record.fail "Environment name '%{name}' is invalid: starts with dash.

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # Rule 5: Check for trailing dash
    let _ = if std.string.is_match "-$" name then
      std.record.fail "Environment name '%{name}' is invalid: ends with dash.

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # Rule 6: Check for consecutive dashes
    let _ = if std.string.contains "--" name then
      std.record.fail "Environment name '%{name}' is invalid: contains consecutive dashes.

Valid format: lowercase letters, numbers, and dashes only.
Examples: dev, staging, production, e2e-config, test-integration"
    else
      name
    in

    # All validations passed
    name,
}
