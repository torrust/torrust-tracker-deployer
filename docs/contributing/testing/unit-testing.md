# Unit Test Naming Style

Unit tests should use descriptive, behavior-driven naming with the `it_should_` prefix instead of the generic `test_` prefix.

## Naming Convention

- **Format**: `it_should_{describe_expected_behavior}`
- **Style**: Use lowercase with underscores, be descriptive and specific
- **Focus**: Describe what the test validates, not just what function it calls

## Examples

### ✅ Good Test Names

```rust
#[test]
fn it_should_create_ansible_host_with_valid_ipv4() {
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let host = AnsibleHost::new(ip);
    assert_eq!(host.as_ip_addr(), &ip);
}

#[test]
fn it_should_fail_with_invalid_ip_address() {
    let result = AnsibleHost::from_str("invalid.ip.address");
    assert!(result.is_err());
}

#[test]
fn it_should_serialize_to_json() {
    let host = AnsibleHost::from_str("192.168.1.1").unwrap();
    let json = serde_json::to_string(&host).unwrap();
    assert_eq!(json, "\"192.168.1.1\"");
}
```

### ❌ Avoid These Test Names

```rust
#[test]
fn test_new() { /* ... */ }

#[test]
fn test_from_str() { /* ... */ }

#[test]
fn test_serialization() { /* ... */ }
```

## Benefits

- **Clarity**: Test names clearly describe the expected behavior
- **Documentation**: Tests serve as living documentation of the code's behavior
- **BDD Style**: Follows Behavior-Driven Development naming conventions
- **Maintainability**: Easier to understand test failures and purpose
