//! Compiled process-boundary regression for #71 containment-capacity input paths.
//! These tests prove static ancestor admission, not race-free directory-handle
//! confinement or production provider authorization.

#[cfg(unix)]
mod unix_path_tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process::{self, Command};

    const BASELINE: &str = include_str!("../../fixtures/containment-capacity-spec2.json");
    const EXECUTABLE: &str = env!("CARGO_BIN_EXE_containment_capacity");

    #[test]
    fn real_fixture_passes_but_outer_and_inner_symlinked_parents_fail() {
        let root = std::env::temp_dir().join(format!(
            "free-energy-capacity-ancestor-cli-{}",
            process::id()
        ));
        let real = root.join("real");
        let nested = real.join("nested");
        fs::create_dir_all(&nested).expect("create isolated fixture parents");
        let fixture = nested.join("capacity.json");
        fs::write(&fixture, BASELINE).expect("write historical fixture");

        let accepted = Command::new(EXECUTABLE)
            .arg(&fixture)
            .output()
            .expect("invoke compiled fixture CLI on real directory");
        assert!(accepted.status.success(), "real fixture failed: {accepted:?}");
        assert!(
            String::from_utf8_lossy(&accepted.stdout).contains("17 passed"),
            "missing historical positive controls: {accepted:?}"
        );
        assert!(accepted.stderr.is_empty());

        let outer = root.join("linked-real");
        symlink(&real, &outer).expect("create outer ancestor alias");
        let inner = real.join("linked-nested");
        symlink(&nested, &inner).expect("create inner ancestor alias");
        for aliased in [
            outer.join("nested/capacity.json"),
            inner.join("capacity.json"),
        ] {
            let rejected = Command::new(EXECUTABLE)
                .arg(&aliased)
                .output()
                .expect("invoke compiled fixture CLI on aliased directory");
            assert!(
                !rejected.status.success(),
                "aliased ancestor was accepted: {rejected:?}"
            );
            assert!(
                rejected.stdout.is_empty(),
                "failed fixture emitted positive result: {rejected:?}"
            );
            assert!(
                String::from_utf8_lossy(&rejected.stderr)
                    .contains("symbolic-link ancestor"),
                "missing deterministic ancestor diagnostic: {rejected:?}"
            );
        }

        fs::remove_dir_all(&root).expect("remove isolated fixture tree");
    }
}
