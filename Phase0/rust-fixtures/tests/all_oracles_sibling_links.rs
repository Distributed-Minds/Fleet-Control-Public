//! Compiled-process regression for the offline #71 aggregate oracle runner.
//! A symlink to a caller-authored executable must not mint an aggregate PASS.
//! This static check does not attest direct executable bytes or defeat races.

#[cfg(unix)]
mod unix_process_tests {
    use std::fs;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::{Path, PathBuf};
    use std::process::{self, Command, Output};

    const RUNNER: &str = env!("CARGO_BIN_EXE_all_oracles");
    const REAL_SIBLING: &str = env!("CARGO_BIN_EXE_containment_capacity");

    fn repository_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Phase0 directory")
            .parent()
            .expect("repository root")
            .to_path_buf()
    }

    fn run(runner: &Path, root: &Path) -> Output {
        Command::new(runner)
            .args(["--family", "containment_capacity", "--root"])
            .arg(root)
            .output()
            .expect("execute isolated compiled aggregate runner")
    }

    #[test]
    fn genuine_direct_sibling_passes_but_symlinked_fake_receipt_is_rejected() {
        let scratch = std::env::temp_dir().join(format!(
            "free-energy-all-oracles-sibling-link-{}",
            process::id()
        ));
        let bin = scratch.join("bin");
        fs::create_dir_all(&bin).expect("create isolated executable directory");

        let runner = bin.join("all_oracles");
        let sibling = bin.join("containment_capacity");
        fs::copy(RUNNER, &runner).expect("copy real compiled aggregate runner");
        fs::copy(REAL_SIBLING, &sibling).expect("copy real compiled sibling");

        let positive = run(&runner, &repository_root());
        assert!(positive.status.success(), "direct sibling failed: {positive:?}");
        assert_eq!(positive.stdout, b"PASS containment_capacity\n");
        assert!(positive.stderr.is_empty(), "{positive:?}");

        fs::remove_file(&sibling).expect("remove direct executable");
        let fake = scratch.join("forged-success.sh");
        fs::write(
            &fake,
            b"#!/bin/sh\nprintf 'containment-capacity fixtures (Rust): 17 passed\\n'\n",
        )
        .expect("write counterfeit success-output executable");
        let mut permissions = fs::metadata(&fake).expect("script metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake, permissions).expect("make counterfeit executable");
        symlink(&fake, &sibling).expect("substitute symlinked sibling");
        assert!(sibling.is_file(), "legacy is_file() follows the fake symlink");
        assert!(
            fs::symlink_metadata(&sibling)
                .expect("symlink metadata")
                .file_type()
                .is_symlink()
        );

        let negative = run(&runner, &repository_root());
        assert!(!negative.status.success(), "symlink was admitted: {negative:?}");
        assert!(negative.stdout.is_empty(), "false aggregate PASS: {negative:?}");
        let stderr = String::from_utf8(negative.stderr).expect("UTF-8 diagnostic");
        assert!(
            stderr.contains("compiled sibling executable is missing, nonregular or symlinked"),
            "missing specific rejection: {stderr:?}"
        );

        fs::remove_dir_all(&scratch).expect("remove isolated test executables");
    }
}
