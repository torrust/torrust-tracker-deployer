use super::{create_environment, deployer_in_temp_dir};

#[test]
fn it_should_list_environments_in_workspace() {
    let (deployer, _workspace) = deployer_in_temp_dir();

    create_environment(&deployer, "sdk-test-list-a");
    create_environment(&deployer, "sdk-test-list-b");

    let env_list = deployer.list().expect("list failed");

    assert_eq!(env_list.total_count, 2);
    let names: Vec<&str> = env_list
        .environments
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert!(names.contains(&"sdk-test-list-a"));
    assert!(names.contains(&"sdk-test-list-b"));
    assert!(!env_list.has_failures());
}

#[test]
fn it_should_return_empty_list_when_no_environments_exist() {
    let (deployer, workspace) = deployer_in_temp_dir();

    // The list command requires the data/ directory to exist.
    std::fs::create_dir_all(workspace.path().join("data"))
        .expect("Failed to create data directory");

    let env_list = deployer.list().expect("list failed");

    assert_eq!(env_list.total_count, 0);
    assert!(env_list.is_empty());
}
