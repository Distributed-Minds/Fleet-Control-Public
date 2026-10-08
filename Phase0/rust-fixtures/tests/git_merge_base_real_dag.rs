//! Executable Git-history evidence for the Phase0 merge-base migration (#71).
//!
//! Unlike the JSON-only topology oracle, this test constructs actual Git
//! commits and asks Git to discover every best base. It is not proof of a
//! general integration planner, virtual-base tree, or Python/Rust parity.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

struct ScratchGit {
    directory: PathBuf,
}

impl ScratchGit {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "free-energy-real-git-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&directory).expect("create isolated Git test directory");
        let repo = Self { directory };
        repo.checked(&["init", "-q"], "", false);
        repo
    }

    fn execute(&self, args: &[&str], input: &str, use_replacements: bool) -> Output {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&self.directory)
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                self.directory.join("unused-global-git-config"),
            )
            .env("GIT_AUTHOR_NAME", "FREE ENERGY Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "FREE ENERGY Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00 +0000")
            .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00 +0000")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if use_replacements {
            command.env_remove("GIT_NO_REPLACE_OBJECTS");
        } else {
            command.env("GIT_NO_REPLACE_OBJECTS", "1");
        }
        let mut child = command.spawn().expect("Git executable required for topology test");
        child
            .stdin
            .take()
            .expect("Git stdin")
            .write_all(input.as_bytes())
            .expect("write Git command stdin");
        child.wait_with_output().expect("collect Git output")
    }

    fn checked(&self, args: &[&str], input: &str, use_replacements: bool) -> String {
        let output = self.execute(args, input, use_replacements);
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .expect("Git object IDs are UTF-8")
            .trim()
            .to_owned()
    }

    fn commit(&self, tree: &str, parents: &[&str], message: &str) -> String {
        let mut args = vec!["commit-tree", tree];
        for parent in parents {
            args.push("-p");
            args.push(parent);
        }
        self.checked(&args, &format!("{message}\n"), false)
    }

    fn merge_bases(&self, left: &str, right: &str, use_replacements: bool) -> BTreeSet<String> {
        self.checked(
            &["merge-base", "--all", left, right],
            "",
            use_replacements,
        )
        .lines()
        .map(str::to_owned)
        .collect()
    }
}

impl Drop for ScratchGit {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn real_git_criss_cross_has_two_best_bases_and_overlays_change_the_answer() {
    let git = ScratchGit::new();
    let empty_tree = git.checked(&["hash-object", "-w", "-t", "tree", "--stdin"], "", false);
    let root = git.commit(&empty_tree, &[], "root");
    let a = git.commit(&empty_tree, &[&root], "a");
    let b = git.commit(&empty_tree, &[&root], "b");
    let x = git.commit(&empty_tree, &[&a, &b], "x");
    let y = git.commit(&empty_tree, &[&b, &a], "y");
    let unrelated = git.commit(&empty_tree, &[], "unrelated-root");

    // Both A and B are best common ancestors, not a single arbitrary base.
    let expected: BTreeSet<String> = [a.clone(), b.clone()].into_iter().collect();
    assert_eq!(expected.len(), 2);
    assert_eq!(git.merge_bases(&x, &y, false), expected);
    assert_eq!(git.merge_bases(&y, &x, false), expected);

    // The same repository has a unique case and a genuinely disconnected case.
    assert_eq!(git.merge_bases(&a, &x, false), [a.clone()].into());
    let disconnected = git.execute(&["merge-base", "--all", &x, &unrelated], "", false);
    assert_eq!(disconnected.status.code(), Some(1));
    assert!(disconnected.stdout.is_empty());

    // Git replacement refs change the effective history view. The exact
    // original object IDs remain equal; raw IDs alone do not bind ancestry.
    git.checked(&["replace", "--graft", &x, &a], "", true);
    assert_eq!(git.merge_bases(&x, &y, true), [a].into());
    assert_eq!(git.merge_bases(&x, &y, false), expected);
}
