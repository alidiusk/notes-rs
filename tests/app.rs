use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

struct TestNotesFile {
    path: PathBuf,
    _tempdir: TempDir,
}

impl TestNotesFile {
    pub fn new() -> anyhow::Result<Self> {
        let tempdir = tempdir()?;
        let path = tempdir.path().to_owned().join("test");

        Ok(TestNotesFile {
            path,
            _tempdir: tempdir,
        })
    }

    pub fn path_as_os_str(&self) -> &OsStr {
        self.path.as_os_str()
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

#[test]
fn init_notes_file() -> anyhow::Result<()> {
    let file = TestNotesFile::new()?;

    let mut cmd = Command::cargo_bin("notes")?;
    cmd.arg("--path").arg(file.path_as_os_str());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("There are no notes."));

    Ok(())
}

// Tests to add:
// - Adding a note.
// - Deleting a note.
// - Getting a note.
// - Editing a note.
// - Errors on incorrect usage.
