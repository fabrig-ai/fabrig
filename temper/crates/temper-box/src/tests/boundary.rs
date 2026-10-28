use jig_boundary::manifest::manifests;
use skein_lib::List;
#[test]
fn the_box_depends_on_no_temper_crate_but_the_setup() {
    let root = std::path::Path::new("../..").canonicalize().expect("temper's root");
    let mut found = false;
    for manifest in &manifests(&root) {
        if manifest.name.as_deref() == Some("temper-box") {
            found = true;
            for dependency in &manifest.dependencies {
                if dependency.starts_with("temper-") {
                    assert_eq!(dependency, "temper-forge-setup", "no other temper crate");
                }
            }
        }
    }
    assert!(found, "temper-box's manifest is found");
}
fn allowed(path: &str) -> bool {
    let Some(path) = path.trim_start().strip_prefix("::") else {
        return false;
    };
    let path = path.trim_start();
    if let Some(group) = path.strip_prefix('{') {
        let Some(end) = group.find('}') else {
            return false;
        };
        for item in group[..end].split(',') {
            let item = item.trim();
            if !["Step", "steps", "Steps", "Group", "Stage", "Kind", ""].contains(&item) {
                return false;
            }
        }
        return true;
    }
    for name in ["Steps", "Group", "Stage", "Kind", "Step", "steps"] {
        if let Some(rest) = path.strip_prefix(name) {
            let rest = rest.trim_start();
            return rest.starts_with(';')
                || rest.starts_with('(')
                || rest.starts_with('{')
                || rest.starts_with(',')
                || rest.starts_with(')')
                || rest.starts_with("::");
        }
    }
    false
}
fn check_source(source: &str) {
    for (position, _) in source.match_indices("temper_forge_setup") {
        let after = position.checked_add("temper_forge_setup".len()).expect("source position");
        assert!(allowed(&source[after..]), "the box names only Step and steps: {}", &source[position..]);
    }
}
#[test]
fn the_box_names_only_steps_and_step_of_the_setup() {
    let mut pending = List::with_capacity(128);
    pending.push(std::path::PathBuf::from("src")).expect("source directory");
    for index in 0..128 {
        let Some(directory) = pending.get(index).cloned() else {
            break;
        };
        for entry in std::fs::read_dir(directory).expect("read source directory") {
            let path = entry.expect("source entry").path();
            if path.is_dir() {
                pending.push(path).expect("bounded source directories");
            } else if path.extension() == Some(std::ffi::OsStr::new("rs"))
                && path.file_name() != Some(std::ffi::OsStr::new("boundary.rs"))
            {
                check_source(&std::fs::read_to_string(path).expect("read source file"));
            }
        }
    }
}
