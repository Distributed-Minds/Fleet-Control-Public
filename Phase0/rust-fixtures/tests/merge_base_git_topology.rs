//! Real Git DAG regression coverage for Phase0 merge-base-topology migration (#71).
//!
//! Unlike the JSON model oracle, these tests construct actual Git commit
//! objects and ask Git for its complete set of best common ancestors.
//! They require a working Git executable; absence is a test failure, not
//! a silently skipped acceptance condition. No ref or external repo is moved.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_REPO: AtomicU64 = AtomicU64::new(0);

struct TestRepo {
    path: PathBuf,
    empty_tree: String,
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

impl TestRepo {
    fn new() -> Self {
        let counter = NEXT_REPO.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "free-energy-merge-base-git-{}-{counter}",
            std::process::id()
        ));
        // Never delete an existing directory that another process could own.
        fs::create_dir(&path).expect("unique test repository path must not exist");
        let mut repo = Self {
            path,
            empty_tree: String::new(),
        };
        repo.checked(&["init", "--bare", "--quiet"], b"");
        repo.empty_tree = repo.checked(&["mktree"], b"");
        assert_eq!(repo.empty_tree.len(), 40);
        repo
    }

    fn output(&self, args: &[&str], input: &[u8]) -> Output {
        let mut child = Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .arg("-c")
            .arg("commit.gpgsign=false")
            .args(args)
            .env("GIT_AUTHOR_NAME", "Topology Fixture")
            .env("GIT_AUTHOR_EMAIL", "topology@example.invalid")
            .env("GIT_COMMITTER_NAME", "Topology Fixture")
            .env("GIT_COMMITTER_EMAIL", "topology@example.invalid")
            .env("GIT_AUTHOR_DATE", "1700000000 +0000")
            .env("GIT_COMMITTER_DATE", "1700000000 +0000")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("git executable required for real topology acceptance");
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(input)
            .expect("write Git fixture input");
        child.wait_with_output().expect("wait for Git")
    }

    fn checked(&self, args: &[&str], input: &[u8]) -> String {
        let result = self.output(args, input);
        assert!(
            result.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout)
            .expect("Git object IDs are ASCII")
            .trim()
            .to_owned()
    }

    fn commit(&self, message: &str, parents: &[&str]) -> String {
        let mut args = vec!["commit-tree", self.empty_tree.as_str()];
        for parent in parents {
            args.extend(["-p", parent]);
        }
        self.checked(&args, format!("{message}\n").as_bytes())
    }

    fn best_bases(&self, left: &str, right: &str) -> Vec<String> {
        let result = self.output(&["merge-base", "--all", left, right], b"");
        let text = String::from_utf8(result.stdout).expect("Git object IDs are ASCII");
        // Git exits 1 (no common ancestor) with no output. Any other failure
        // is a real test failure, not an inferred empty base set.
        if result.status.code() == Some(1) && text.is_empty() {
            return Vec::new();
        }
        assert!(
            result.status.success(),
            "merge-base failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut bases: Vec<String> = text.lines().map(str::to_owned).collect();
        bases.sort();
        assert!(bases.iter().all(|s| s.len() == 40));
        bases
    }
}

#[test]
fn criss_cross_has_two_best_bases_not_one_arbitrarily_chosen_base() {
    let repo = TestRepo::new();
    let root = repo.commit("root", &[]);
    let left = repo.commit("left", &[&root]);
    let right = repo.commit("right", &[&root]);
    // Both tips merge the same incomparable ancestors, in opposite order.
    let tip_a = repo.commit("tip-a", &[&left, &right]);
    let tip_b = repo.commit("tip-b", &[&right, &left]);

    let mut expected = vec![left, right];
    expected.sort();
    assert_ne!(expected[0], expected[1]);
    assert_eq!(repo.best_bases(&tip_a, &tip_b), expected);
    assert_eq!(repo.best_bases(&tip_b, &tip_a), expected);
}

#[test]
fn a_single_best_base_is_not_misclassified_as_multiple() {
    let repo = TestRepo::new();
    let root = repo.commit("root", &[]);
    let common = repo.commit("common", &[&root]);
    let left = repo.commit("left", &[&common]);
    let right = repo.commit("right", &[&common]);
    let further_left = repo.commit("further-left", &[&left]);
    assert_eq!(repo.best_bases(&further_left, &right), vec![common]);
    assert_eq!(repo.best_bases(&left, &further_left), vec![left]);
}

#[test]
fn unrelated_histories_have_no_merge_base() {
    let repo = TestRepo::new();
    let unrelated_a = repo.commit("unrelated-a", &[]);
    let unrelated_b = repo.commit("unrelated-b", &[]);
    assert_ne!(unrelated_a, unrelated_b);
    assert!(repo.best_bases(&unrelated_a, &unrelated_b).is_empty());
}

#[test]
fn changing_an_observed_tip_changes_the_authoritative_base_set() {
    let repo = TestRepo::new();
    let root = repo.commit("root", &[]);
    let left = repo.commit("left", &[&root]);
    let right = repo.commit("right", &[&root]);
    let tip_a = repo.commit("tip-a", &[&left, &right]);
    let tip_b = repo.commit("tip-b", &[&right, &left]);
    let original = repo.best_bases(&tip_a, &tip_b);
    assert_eq!(original.len(), 2);

    // A new actual merge commit on top of one side makes that side the
    // unique best ancestor. Old multi-base predictions are now stale.
    let advanced = repo.commit("advanced", &[&tip_a, &tip_b]);
    assert_eq!(repo.best_bases(&advanced, &tip_b), vec![tip_b]);
    assert_ne!(repo.best_bases(&advanced, &tip_b), original);
}
