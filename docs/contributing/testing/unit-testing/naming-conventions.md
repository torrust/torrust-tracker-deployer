# Unit Test Naming Style

Unit tests should use descriptive, behavior-driven naming with the `it_should_` prefix instead of the generic `test_` prefix.

## Naming Convention

- **Format**: `it_should_{expected_behavior}_when_{condition}` or `it_should_{expected_behavior}_given_{state}`
- **Style**: Use lowercase with underscores, be descriptive and specific
- **Structure**: Follow the three-part pattern (What-When-Then)

### The Three-Part Structure

Every test name should clearly communicate:

1. **What** - The expected behavior or outcome being tested
2. **When** - The triggering condition or scenario (use `when_` or `given_`)
3. **Context** - Implicit from the test module/struct being tested

This follows established conventions from:

- **Roy Osherove's standard**: `UnitOfWork_StateUnderTest_ExpectedBehavior`
- **BDD Given-When-Then**: Behavior-driven development naming
- **AAA Pattern**: Arrange-Act-Assert reflected in test names

### Key Properties of Good Test Names

A well-named test should tell you:

- ✅ **What behavior** is being validated (the expected outcome)
- ✅ **When it happens** (the triggering condition or preconditions)
- ✅ **What's being tested** (implicit from module context, or explicit in name)
- ✅ **Why it matters** (clear from the behavior description)

### Guidelines

- **Be specific about conditions**: Use `when_`, `given_`, `with_`, or `for_` to describe the scenario
- **Describe behavior, not implementation**: Focus on what happens, not how
- **Use complete phrases**: Test names can be long - clarity beats brevity
- **Include edge cases explicitly**: `when_input_is_empty`, `when_value_exceeds_limit`
- **Describe error conditions clearly**: `when_file_not_found`, `given_invalid_format`

## Examples

### ✅ Good Test Names (Following Three-Part Structure)

```rust
#[test]
fn it_should_create_valid_host_when_given_ipv4_address() {
    // What: create valid host | When: given IPv4 address
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let host = AnsibleHost::new(ip);
    assert_eq!(host.as_ip_addr(), &ip);
}

#[test]
fn it_should_return_error_when_parsing_invalid_ip_string() {
    // What: return error | When: parsing invalid IP string
    let result = AnsibleHost::from_str("invalid.ip.address");
    assert!(result.is_err());
}

#[test]
fn it_should_serialize_to_json_string_when_valid_host_exists() {
    // What: serialize to JSON | When: valid host exists
    let host = AnsibleHost::from_str("192.168.1.1").unwrap();
    let json = serde_json::to_string(&host).unwrap();
    assert_eq!(json, "\"192.168.1.1\"");
}

#[test]
fn it_should_reject_environment_name_when_containing_uppercase_letters() {
    // What: reject name | When: containing uppercase
    let result = EnvironmentName::new("MyEnv".to_string());
    assert!(matches!(result, Err(ValidationError::InvalidCharacters)));
}

#[test]
fn it_should_preserve_order_when_deserializing_from_json() {
    // What: preserve order | When: deserializing from JSON
    let json = r#"{"first": 1, "second": 2}"#;
    let config: Config = serde_json::from_str(json).unwrap();
    assert_eq!(config.keys().collect::<Vec<_>>(), vec!["first", "second"]);
}
```

### ❌ Avoid These Test Names

```rust
// ❌ Too generic - doesn't describe behavior or condition
#[test]
fn test_new() { /* ... */ }

#[test]
fn test_from_str() { /* ... */ }

#[test]
fn test_serialization() { /* ... */ }

// ❌ Focuses on implementation, not behavior
#[test]
fn it_should_call_validate_method() { /* ... */ }

#[test]
fn it_should_use_serde_deserialize() { /* ... */ }

// ❌ Missing condition/scenario context
#[test]
fn it_should_fail() { /* What fails? When? Why? */ }

#[test]
fn it_should_return_value() { /* Which value? Under what conditions? */ }

// ❌ Too vague about expected behavior
#[test]
fn it_should_work_correctly() { /* What does "work correctly" mean? */ }

#[test]
fn it_should_handle_edge_case() { /* Which edge case? How is it handled? */ }
```

### Pattern Examples by Category

#### Success Cases

```rust
it_should_create_environment_when_given_valid_name()
it_should_return_formatted_output_when_data_is_complete()
it_should_preserve_state_when_serializing_and_deserializing()
```

#### Error Cases

```rust
it_should_return_error_when_file_does_not_exist()
it_should_reject_config_when_required_field_is_missing()
it_should_fail_validation_when_port_exceeds_maximum()
```

#### Edge Cases

```rust
it_should_handle_empty_string_when_parsing_optional_field()
it_should_return_default_when_environment_variable_is_unset()
it_should_allow_maximum_length_when_validating_string_input()
```

#### State Transitions

```rust
it_should_transition_to_active_when_provisioning_completes()
it_should_remain_in_pending_state_when_validation_fails()
it_should_rollback_to_previous_state_when_operation_errors()
```

## Common Anti-Patterns to Avoid

### ❌ Testing Implementation Details

```rust
// Bad: Tests how something is done
it_should_call_repository_save_method()

// Good: Tests what happens
it_should_persist_environment_when_creation_succeeds()
```

### ❌ Vague Conditions

```rust
// Bad: Unclear when this happens
it_should_fail_validation()

// Good: Specific condition
it_should_fail_validation_when_port_number_is_negative()
```

### ❌ Missing Expected Outcome

```rust
// Bad: Doesn't state what happens
it_should_process_input_when_valid()

// Good: Clear outcome
it_should_return_parsed_config_when_input_is_valid_json()
```

### ❌ Technical Jargon Without Context

```rust
// Bad: Requires domain knowledge to understand
it_should_deserialize_dto()

// Good: Explains the behavior
it_should_convert_json_to_environment_config_when_deserializing()
```

## Best Practices

### Do's ✅

- **Use complete phrases**: `when_given_empty_string` not `when_empty`
- **Be explicit about data states**: `when_file_does_not_exist` not `when_no_file`
- **Describe outcomes clearly**: `should_return_error` not `should_fail`
- **Include relevant values**: `when_port_exceeds_65535` not `when_port_too_large`
- **Name error types**: `should_return_validation_error` not `should_return_error`

### Don'ts ❌

- **Don't test methods directly**: Test behaviors, not function names
- **Don't use abbreviations**: `when_cfg_invalid` → `when_configuration_is_invalid`
- **Don't skip the condition**: Always include the `when_` or `given_` clause
- **Don't be overly technical**: Focus on business behavior, not implementation
- **Don't make assumptions**: Be explicit about preconditions

## Reading Your Test Names

A good test name should read naturally as a sentence when you add spaces:

- ❌ `test_parse_error` → "test parse error" (unclear)
- ✅ `it_should_return_error_when_parsing_invalid_json` → "it should return error when parsing invalid JSON" (clear)

## Benefits

- **Clarity**: Test names clearly describe the expected behavior and conditions
- **Documentation**: Tests serve as living, executable specifications of behavior
- **BDD Style**: Follows established Behavior-Driven Development conventions
- **Maintainability**: Easier to understand test failures and identify affected behavior
- **Traceability**: Clear mapping between requirements/behaviors and tests
- **Debugging**: Failed test names immediately tell you what broke and under what conditions
- **Code Reviews**: Reviewers can understand test purpose without reading implementation

## Quick Reference

### Test Name Template

```rust
it_should_{expected_behavior}_when_{triggering_condition}
it_should_{expected_behavior}_given_{initial_state}
```

### Checklist for Good Test Names

- [ ] Describes the **expected behavior** clearly
- [ ] Specifies the **triggering condition** or scenario
- [ ] Uses complete phrases, not abbreviations
- [ ] Reads naturally as a sentence
- [ ] Focuses on **what** happens, not **how**
- [ ] Includes specific values or states when relevant
- [ ] Is specific enough to understand without reading the test body

## References and Further Reading

This guide is based on established testing conventions and best practices:

- **Roy Osherove's Naming Standard**: "The Art of Unit Testing" - The three-part naming pattern: `UnitOfWork_StateUnderTest_ExpectedBehavior`
- **BDD (Behavior-Driven Development)**: Dan North's Given-When-Then pattern for describing behavior
- **AAA Pattern**: Arrange-Act-Assert structure reflected in test organization and naming
- **Google Testing Blog**: Best practices for test naming and structure
- **Martin Fowler**: Testing patterns and behavior-focused naming
- **Kent Beck**: Test-Driven Development principles

### External Resources

- [The Art of Unit Testing (Roy Osherove)](https://www.artofunittesting.com/)
- [BDD Fundamentals (Dan North)](https://dannorth.net/introducing-bdd/)
- [Google Testing Blog](https://testing.googleblog.com/)
- [xUnit Test Patterns](http://xunitpatterns.com/)
- [Growing Object-Oriented Software, Guided by Tests](http://www.growing-object-oriented-software.com/)
