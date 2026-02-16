# Test Builders for Commands

Commands should provide test builders for simplified unit testing.

## Example

```rust
use torrust_tracker_deployer::application::commands::destroy::tests::DestroyCommandTestBuilder;

#[test]
fn it_should_create_destroy_command_with_all_dependencies() {
    let (command, _temp_dir) = DestroyCommandTestBuilder::new().build();

    // Verify the command was created
    assert_eq!(Arc::strong_count(&command.opentofu_client), 1);
}
```

## Benefits of Test Builders

- Manages `TempDir` lifecycle automatically
- Provides sensible defaults for all dependencies
- Allows selective customization of dependencies
- Returns only the command and necessary test artifacts
