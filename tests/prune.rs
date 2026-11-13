use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "symlonk-prune-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_prune(tmp: &Path) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_symlonk"))
        .arg("create")
        .arg("links")
        .arg(tmp.join("config.toml"))
        .arg("--prune")
        .arg("-l")
        .arg(tmp.join("lock.toml"))
        .current_dir(tmp)
        .output()
        .unwrap();

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), combined)
}

#[test]
fn prune_drops_stale_entry_whose_path_is_already_gone() {
    let tmp = TestDir::new();
    let src = tmp.join("src");
    let dst = tmp.join("dst");
    let file = src.join("file");
    let gone = dst.join("gone");
    let present = dst.join("present");
    let lock = tmp.join("lock.toml");

    fs::create_dir_all(&src).unwrap();
    fs::create_dir_all(&dst).unwrap();
    fs::write(&file, "").unwrap();
    symlink(&file, &present).unwrap();

    fs::write(
        tmp.join("config.toml"),
        format!(
            "[config]\nsource_dir = \"{}\"\ndestination_dir = \"{}\"\n",
            src.display(),
            dst.display()
        ),
    )
    .unwrap();
    fs::write(
        &lock,
        format!(
            "[symlinks]\n\"{}\" = \"{}\"\n\"{}\" = \"{}\"\n",
            gone.display(),
            src.join("old").display(),
            present.display(),
            file.display()
        ),
    )
    .unwrap();

    let (success, output) = run_prune(&tmp.0);
    assert!(success, "first run failed:\n{output}");
    assert!(
        !output.contains("ERROR"),
        "first run logged an error:\n{output}"
    );

    let lock_contents = fs::read_to_string(&lock).unwrap();
    assert!(
        !lock_contents.contains("gone"),
        "stale missing entry was kept in the lock file:\n{lock_contents}"
    );
    assert!(
        !lock_contents.contains("present"),
        "stale present entry was kept in the lock file:\n{lock_contents}"
    );
    assert!(
        fs::symlink_metadata(&present).is_err(),
        "existing stale symlink was not unlinked"
    );

    let (success, output) = run_prune(&tmp.0);
    assert!(success, "second run failed:\n{output}");
    assert!(
        !output.contains("ERROR"),
        "second run logged an error:\n{output}"
    );
    assert_eq!(
        fs::read_to_string(&lock).unwrap(),
        lock_contents,
        "second run changed the already-pruned lock file"
    );
}
