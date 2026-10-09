//! Historical real-Git merge-base differential probes, ported to compiled Rust.
//! Complements the model-only merge_base_topology oracle; no provider mutation.
//!
//! The source Python checker constructs this DAG and checks a unique merge base,
//! a criss-cross with two best bases, a shallow-history cutoff, and a replacement
//! ancestry cutoff. This test runs actual Git on a disposable local repository.

use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_REPO: AtomicU64 = AtomicU64::new(0);

struct GitRepo {
    root: PathBuf,
}

struct GitOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

impl GitRepo {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("wall clock after Unix epoch")
            .as_nanos();
        let serial = NEXT_REPO.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "free-energy-real-git-topology-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh disposable Git repository path");
        let repo = Self { root };
        repo.must(&["init", "-q"], "");
        repo
    }

    fn run(&self, args: &[&str], input: &str) -> GitOutput {
        let mut command = Command::new("git");
        command.arg("-C").arg(&self.root).args(args);
        // Ignore inherited Git checkout/CI overrides that can silently
        // redirect objects or mask replacement refs. No network Git command
        // is executed, and all writes remain inside our disposable directory.
        for key in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_COMMON_DIR",
            "GIT_NAMESPACE",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_CONFIG_COUNT",
            "GIT_CONFIG_PARAMETERS",
            "GIT_NO_REPLACE_OBJECTS",
        ] {
            command.env_remove(key);
        }
        command
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_AUTHOR_NAME", "FREE ENERGY topology fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "FREE ENERGY topology fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00+00:00")
            .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00+00:00")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().expect("Git executable required by topology CI");
        child
            .stdin
            .take()
            .expect("Git child stdin")
            .write_all(input.as_bytes())
            .expect("write deterministic Git fixture input");
        let result = child.wait_with_output().expect("wait for local Git");
        GitOutput {
            success: result.status.success(),
            stdout: String::from_utf8(result.stdout)
                .expect("Git object identity output is UTF-8")
                .trim()
                .to_owned(),
            stderr: String::from_utf8_lossy(&result.stderr).into_owned(),
        }
    }

    fn must(&self, args: &[&str], input: &str) -> String {
        let output = self.run(args, input);
        assert!(
            output.success,
            "Git {args:?} failed in disposable fixture: {}",
            output.stderr
        );
        output.stdout
    }

    fn commit(&self, tree: &str, message: &str, parents: &[&str]) -> String {
        let mut arguments = vec!["commit-tree", tree, "-m", message];
        for parent in parents {
            arguments.extend(["-p", parent]);
        }
        self.must(&arguments, "")
    }

    fn git_dir(&self) -> PathBuf {
        let raw = self.must(&["rev-parse", "--git-dir"], "");
        let git_dir = Path::new(&raw);
        if git_dir.is_absolute() {
            git_dir.to_path_buf()
        } else {
            self.root.join(git_dir)
        }
    }
}

impl Drop for GitRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn real_git_unique_criss_cross_shallow_and_replacement_probes() {
    let repo = GitRepo::new();
    let tree = repo.must(&["mktree"], "");
    let root = repo.commit(&tree, "root", &[]);
    let left = repo.commit(&tree, "left", &[&root]);
    let right = repo.commit(&tree, "right", &[&root]);
    let unique_tip = repo.commit(&tree, "unique-tip", &[&left]);

    // Historical real-unique-base: exactly the common root.
    let unique = repo.run(&["merge-base", "--all", &unique_tip, &right], "");
    assert!(unique.success, "unique merge-base failed: {}", unique.stderr);
    assert_eq!(unique.stdout.lines().collect::<Vec<_>>(), vec![root.as_str()]);

    // Historical real-criss-cross: two incomparable best common ancestors.
    let merge_left = repo.commit(&tree, "merge-left", &[&left, &right]);
    let merge_right = repo.commit(&tree, "merge-right", &[&right, &left]);
    let criss = repo.run(&["merge-base", "--all", &merge_left, &merge_right], "");
    assert!(criss.success, "criss-cross merge-base failed: {}", criss.stderr);
    let observed = criss.stdout.lines().collect::<BTreeSet<_>>();
    assert_eq!(observed, BTreeSet::from([left.as_str(), right.as_str()]));
    assert_eq!(criss.stdout.lines().count(), 2, "duplicate merge bases");

    // Historical real-shallow-boundary: a cut ancestor must not be treated
    // as a fully observed common base merely because an object exists.
    let shallow = repo.git_dir().join("shallow");
    fs::write(&shallow, format!("{left}\n")).expect("install local shallow boundary");
    let cut = repo.run(&["merge-base", "--all", &unique_tip, &right], "");
    fs::remove_file(&shallow).expect("remove disposable shallow boundary");
    assert!(
        !cut.success && cut.stdout.is_empty(),
        "shallow history retained a false complete merge base: {:?}",
        cut.stdout
    );

    // Historical real-replacement-ancestry: an authorized replacement ref
    // changes observed ancestry, invalidating the previously cached base.
    let replacement_root = repo.commit(&tree, "replacement-root", &[]);
    let replacement_left = repo.commit(&tree, "replacement-left", &[&replacement_root]);
    repo.must(&["replace", &left, &replacement_left], "");
    let replaced = repo.run(&["merge-base", "--all", &unique_tip, &right], "");
    assert!(
        !replaced.success && replaced.stdout.is_empty(),
        "replacement ancestry retained a stale base: {:?}",
        replaced.stdout
    );

    // Positive recovery after removing the replacement proves the denial was
    // due to the changed ancestry, not a broken repository/test fixture.
    repo.must(&["replace", "-d", &left], "");
    let restored = repo.run(&["merge-base", "--all", &unique_tip, &right], "");
    assert!(restored.success, "restored Git history must be readable");
    assert_eq!(restored.stdout, root);
}
