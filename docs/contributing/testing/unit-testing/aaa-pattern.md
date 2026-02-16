# AAA Pattern (Arrange-Act-Assert)

All tests should follow the AAA pattern, also known as Given-When-Then:

- **Arrange (Given)**: Set up the test data and preconditions
- **Act (When)**: Execute the behavior being tested
- **Assert (Then)**: Verify the expected outcome

This pattern makes tests:

- Easy to read and understand
- Clear about what is being tested
- Simple to maintain and modify

## Example

```rust
#[test]
fn it_should_create_ansible_host_with_valid_ipv4() {
    // Arrange: Set up test data
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Act: Execute the behavior
    let host = AnsibleHost::new(ip);

    // Assert: Verify the outcome
    assert_eq!(host.as_ip_addr(), &ip);
}
```

## Benefits

- **Clarity**: Each section has a clear purpose
- **Structure**: Consistent test organization across the codebase
- **Debugging**: Easy to identify which phase is failing
- **Maintenance**: Simple to modify specific parts of the test
