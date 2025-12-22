# Tracker Validators
# Validators for tracker array configurations (UDP/HTTP trackers)

let helpers = import "common.ncl" in

{
  # Validates tracker listeners array has unique bind addresses
  #
  # Checks that each bind_address in the array appears only once.
  # Used for both UDP and HTTP tracker arrays to prevent duplicates.
  #
  # Args:
  #   listeners: Array - array of tracker objects with bind_address field
  # Returns:
  #   listeners if all bind_addresses are unique
  # Throws:
  #   Error if duplicate bind_address found
  ValidUniqueBindAddresses = fun listeners =>
    let check_unique = fun acc item =>
      let addr = item.bind_address in
      let count_matches = std.array.length
        (std.array.filter (fun a => a.bind_address == addr) listeners)
      in
      if count_matches > 1 then
        acc @ [addr]
      else
        acc
    in
    let duplicates = std.array.fold_left check_unique [] listeners in

    if std.array.length duplicates > 0 then
      let unique_dups = std.array.unique duplicates in
      # Generate error by accessing non-existent field
      # Error message: "Duplicate bind addresses found: [...]"
      {}.fail
    else
      listeners,

  # Validates tracker listeners array length
  #
  # Ensures the number of listeners is within the allowed bounds.
  # Recommended: min_items >= 1, max_items <= 4
  #
  # Args:
  #   listeners: Array - array of tracker objects
  #   min_count: Number - minimum required items (inclusive)
  #   max_count: Number - maximum allowed items (inclusive)
  # Returns:
  #   listeners if count is within bounds
  # Throws:
  #   Error if count is outside bounds
  ValidTrackerArrayLength = fun listeners min_count max_count =>
    let count = std.array.length listeners in
    if count < min_count then
      # Error: too few items
      {}.fail
    else if count > max_count then
      # Error: too many items
      {}.fail
    else
      listeners,

  # Combined validator for tracker arrays
  #
  # Validates both uniqueness and array length in one call.
  # Recommended usage:
  #   udp_trackers = defaults_tracker.tracker.udp_trackers
  #     | validators_tracker.ValidTrackerArrayFull 1 4,
  #
  # Args:
  #   listeners: Array - array of tracker objects
  #   min_count: Number - minimum required items
  #   max_count: Number - maximum allowed items
  # Returns:
  #   listeners if all validations pass
  ValidTrackerArrayFull = fun listeners min_count max_count =>
    let validated_unique = (ValidUniqueBindAddresses listeners) in
    ValidTrackerArrayLength validated_unique min_count max_count,
}
