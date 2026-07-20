use std::process::Command;

#[test]
fn libs_do_not_depend_on_dare_cli() {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let meta: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse cargo metadata json");
    let packages = meta["packages"].as_array().expect("packages array");
    let lib_names = ["dare-core", "dare-contracts", "dare-config", "dare-assets"];
    for pkg in packages {
        let name = pkg["name"].as_str().unwrap_or("");
        if !lib_names.contains(&name) {
            continue;
        }
        let deps = pkg["dependencies"].as_array().cloned().unwrap_or_default();
        for dep in deps {
            let dep_name = dep["name"].as_str().unwrap_or("");
            assert_ne!(dep_name, "dare-cli", "{name} must not depend on dare-cli");
        }
    }
}
