//! The patch command over a git base (data/codec/command.md, 9).

#[cfg(test)]
mod tests {
    use skein_codegen::patch::Tree;
    use skein_scratch::Scratch;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    const V1: &str = "family shelf 1\nrecord Book { id: u8 }\nenum State { ready }\n";
    const V2: &str = "family shelf 2\nrecord Book { id: u8, score: u32 }\nenum State { ready }\n";

    fn crate_files(name: &str, files: &[(&str, &str)]) -> Scratch {
        let root = Scratch::new(&format!("patch-check-{name}"));
        for (path, text) in files {
            let path = root.path().join(path);
            fs::create_dir_all(path.parent().expect("file parent")).expect("directories");
            fs::write(path, text).expect("fixture file");
        }
        root
    }

    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .expect("scratch git");
        assert!(output.status.success(), "git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr));
    }

    #[test]
    fn binary_reads_crate_relative_base_and_returns_all_exit_codes() {
        let root = crate_files("binary", &[("nested/owner/schema/shelf-v1.schema", V1)]);
        git(root.path(), &["init", "--quiet"]);
        git(root.path(), &["add", "."]);
        git(
            root.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                "baseline",
            ],
        );
        let owner = root.path().join("nested/owner");
        let output = Command::new(skein_scratch::paths::binary("skein-codegen"))
            .arg("patch")
            .arg(&owner)
            .args(["--base", "HEAD", "--baseline"])
            .output()
            .expect("binary baseline");
        assert_eq!(output.status.code(), Some(0_i32));
        assert_eq!(String::from_utf8(output.stdout).expect("report utf8"), "shelf: kept version 1, its first\n");
        git(root.path(), &["add", "."]);
        git(
            root.path(),
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "-m",
                "kept",
            ],
        );
        fs::write(owner.join("schema/shelf-v2.schema"), V2).expect("current v2");
        let output = Command::new(skein_scratch::paths::binary("skein-codegen"))
            .arg("patch")
            .arg(&owner)
            .args(["--base", "HEAD", "--owner", "shop"])
            .output()
            .expect("binary patch");
        assert_eq!(output.status.code(), Some(1_i32));
        assert!(String::from_utf8(output.stdout).expect("utf8").contains("Asked"), "planner refusal printed");
        let before = Tree::disk(&owner).landed().expect("tree snapshot");
        let output = Command::new(skein_scratch::paths::binary("skein-codegen"))
            .arg("patch")
            .arg(&owner)
            .args(["--base", "HEAD", "--check"])
            .output()
            .expect("binary check");
        assert_eq!(output.status.code(), Some(1_i32));
        assert_eq!(Tree::disk(&owner).landed().expect("tree snapshot"), before);
        let output =
            Command::new(skein_scratch::paths::binary("skein-codegen")).arg("patch").output().expect("binary error");
        assert_eq!(output.status.code(), Some(2_i32));
        fs::write(owner.join("schema/shelf-v2.schema"), "invalid").expect("invalid current");
        let output = Command::new(skein_scratch::paths::binary("skein-codegen"))
            .arg("patch")
            .arg(&owner)
            .args(["--base", "HEAD", "--check"])
            .output()
            .expect("binary read error");
        assert_eq!(output.status.code(), Some(2_i32));
        assert!(
            String::from_utf8(output.stderr).expect("utf8").contains("schema/shelf-v2.schema"),
            "read error printed"
        );
    }
}
