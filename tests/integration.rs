use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_end_to_end_generation() {
    let fixtures = project_root().join("tests/fixtures");

    // Create temp output directory
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("output.xml");
    let cache_path = temp.path().join(".cache.json");

    // Create config (no more base_class or parent_mappings)
    let config = format!(
        r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "{}"
cache_file = "{}"

[[sources]]
type = "directory"
path = "{}"
"#,
        output_path.display().to_string().replace('\\', "/"),
        cache_path.display().to_string().replace('\\', "/"),
        fixtures.display().to_string().replace('\\', "/"),
    );

    let config_path = temp.path().join("luban.config.toml");
    fs::write(&config_path, &config).unwrap();

    // Create minimal tsconfig
    fs::write(
        temp.path().join("tsconfig.json"),
        r#"{"compilerOptions": {}}"#,
    )
    .unwrap();

    // Run the generator
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_luban-gen"))
        .arg("-c")
        .arg(&config_path)
        .status()
        .expect("Failed to run luban-gen");

    assert!(status.success(), "luban-gen failed");

    // Verify output exists
    assert!(output_path.exists(), "Output file not created");

    // Verify output content
    let output = fs::read_to_string(&output_path).unwrap();

    // Check for SimpleClass (class without extends has no parent)
    assert!(
        output.contains(r#"<bean name="SimpleClass">"#)
            && !output.contains(r#"<bean name="SimpleClass" parent="#),
        "Missing SimpleClass bean (should have no parent)"
    );

    // Check for DamageTrigger (class implements EntityTrigger, has parent="EntityTrigger")
    assert!(
        output.contains(r#"<bean name="DamageTrigger" parent="EntityTrigger">"#),
        "Missing DamageTrigger bean with parent='EntityTrigger'"
    );

    // Check for EntityTrigger (interface, should not have parent)
    assert!(
        output.contains(r#"<bean name="EntityTrigger">"#)
            && !output.contains(r#"<bean name="EntityTrigger" parent="#),
        "Missing EntityTrigger interface bean (should have no parent)"
    );

    // Check for list type
    assert!(
        output.contains(r#"type="list,string""#),
        "Missing list type"
    );

    // Check for map type
    assert!(
        output.contains(r#"type="map,string,double""#),
        "Missing map type"
    );

    // Check for optional field
    assert!(
        output.contains(r#"type="bool?""#),
        "Missing optional bool field"
    );

    // Check for ObjectFactory field with ObjectFactory tag
    assert!(
        output.contains(r#"tags="ObjectFactory=true""#),
        "Missing ObjectFactory=true tag for ObjectFactory field"
    );
}

#[test]
fn test_force_regeneration() {
    let fixtures = project_root().join("tests/fixtures");
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("output.xml");
    let cache_path = temp.path().join(".cache.json");

    let config = format!(
        r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "{}"
cache_file = "{}"

[[sources]]
type = "directory"
path = "{}"
"#,
        output_path.display().to_string().replace('\\', "/"),
        cache_path.display().to_string().replace('\\', "/"),
        fixtures.display().to_string().replace('\\', "/"),
    );

    let config_path = temp.path().join("luban.config.toml");
    fs::write(&config_path, &config).unwrap();
    fs::write(
        temp.path().join("tsconfig.json"),
        r#"{"compilerOptions": {}}"#,
    )
    .unwrap();

    // Run first time
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_luban-gen"))
        .arg("-c")
        .arg(&config_path)
        .status()
        .expect("Failed to run luban-gen");
    assert!(status.success());

    // Run with force flag
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_luban-gen"))
        .arg("-c")
        .arg(&config_path)
        .arg("--force")
        .status()
        .expect("Failed to run luban-gen with --force");
    assert!(status.success());
}
