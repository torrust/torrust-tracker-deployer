//! Workflow test — verifies chained local operations in a single scenario.
//!
//! This mirrors the SDK example in `packages/sdk/examples/basic_usage.rs`:
//! create → list → show → exists → destroy → purge, asserting intermediate
//! state at each step.

use super::{
    assert_environment_exists, assert_environment_not_exists, create_environment,
    deployer_in_temp_dir,
};

#[test]
fn it_should_complete_local_lifecycle_workflow() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    // 1. Create
    let env_name = create_environment(&deployer, "sdk-workflow");

    // 2. Exists — should be true after create
    assert_environment_exists(&deployer, &env_name);

    // 3. Show — inspect the created environment
    let info = deployer.show(&env_name).expect("show failed");
    assert_eq!(info.name, "sdk-workflow");
    assert_eq!(info.state, "Created");
    assert_eq!(info.provider, "LXD");

    // 4. List — the environment should appear
    let env_list = deployer.list().expect("list failed");
    assert_eq!(env_list.total_count, 1);
    assert_eq!(env_list.environments[0].name, "sdk-workflow");
    assert!(!env_list.has_failures());

    // 5. Destroy — transitions state, data remains
    deployer.destroy(&env_name).expect("destroy failed");

    let info = deployer.show(&env_name).expect("show after destroy failed");
    assert_eq!(info.name, "sdk-workflow");
    // State changes after destroy (no longer "Created")
    assert_ne!(info.state, "Created");

    // 6. Purge — removes all local data
    deployer.purge(&env_name).expect("purge failed");

    assert_environment_not_exists(&deployer, &env_name);

    // 7. List — workspace should be empty after purge
    // Note: data/ dir still exists (destroy/purge don't remove it)
    let env_list = deployer.list().expect("list after purge failed");
    assert_eq!(env_list.total_count, 0);
    assert!(env_list.is_empty());
}
