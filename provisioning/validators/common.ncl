# Common Validators
# Utility functions used by all validators

{
  # Validates port number (must be 1-65535, rejects 0 explicitly)
  #
  # Port 0 is explicitly rejected per project ADR:
  # "Port 0 means 'any available port' in most systems, which is unsuitable
  # for production configuration where we need predictable listening addresses."
  #
  # Args:
  #   port: Number - port number to validate
  # Returns:
  #   port if valid
  # Throws:
  #   Error if port is not in valid range
  ValidPort = fun port =>
    let port_num = if std.is_number port then port else std.string.to_number port in
    if port_num >= 1 && port_num <= 65535 then
      port_num
    else
      std.record.fail "Port must be 1-65535 (0 not supported), got %{std.string.from_number port_num}",

  # Validates non-empty string
  #
  # Args:
  #   s: String - string to validate
  # Returns:
  #   s if non-empty
  # Throws:
  #   Error if string is empty
  NonEmptyString = fun s =>
    if std.string.length s > 0 then s
    else std.record.fail "String cannot be empty",

  # Validates bind address format (IP:PORT)
  #
  # Args:
  #   addr: String - address in format "IP:PORT"
  # Returns:
  #   addr if valid
  # Throws:
  #   Error if format is invalid or port is out of range
  ValidBindAddress = fun addr =>
    # Simple regex check for IP:PORT format
    # Pattern: one or more digits/dots followed by colon followed by digits
    let is_valid_format = std.string.is_match "^[0-9.]+:[0-9]+$" addr in

    if !is_valid_format then
      std.record.fail "Bind address must be IP:PORT format, got '%{addr}'"

    else
      # Extract port and validate range
      let parts = std.string.split ":" addr in
      let last_index = (std.array.length parts) - 1 in
      let port_str = std.array.at last_index parts in
      let port_num = std.string.to_number port_str in

      if port_num >= 1 && port_num <= 65535 then
        addr
      else
        std.record.fail "Port in bind address must be 1-65535, got %{port_str}",
}
