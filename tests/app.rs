use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

struct TestCommand {
    pub cmd: Command,
    pub path: PathBuf,
    _tempdir: TempDir,
}

impl TestCommand {
    pub fn new() -> anyhow::Result<Self> {
        let tempdir = tempdir()?;
        let path = tempdir.path().to_owned().join("test");
        let mut cmd = Command::cargo_bin("notes")?;
        cmd.arg("--path").arg(path.as_os_str());

        Ok(TestCommand {
            cmd,
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

macro_rules! cmd_with_args {
    ($cmd:ident, [$($arg:expr),*]) => {{
        let mut new_cmd = Command::cargo_bin("notes")?;
        new_cmd.arg("--path").arg($cmd.path_as_os_str());

        $(
            new_cmd.arg($arg);
        )*

        $cmd.cmd = new_cmd;

        $cmd
    }};
    ($($arg:expr),*) => {{
        let mut cmd = TestCommand::new()?;

        $(
            cmd.cmd.arg($arg);
        )*

        cmd
    }};
}

// Test macro that takes cmd inputs and the resulting predicates,
// then expands to full test code that tests each input against its
// predicate

macro_rules! assert_success {
    ($cmd:ident, $pred:expr) => {
        $cmd.cmd.assert().success().stdout($pred)
    };
}

#[test]
fn init_notes_file() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!();
    assert_success!(cmd, predicate::str::contains("There are no notes."));

    Ok(())
}

#[test]
fn get_all_notes_empty() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("get", "--all");
    assert_success!(cmd, predicate::str::contains("There are no notes."));

    Ok(())
}

#[test]
fn one_new_note() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("new", "test");

    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd, ["get", "--all"]);

    assert_success!(cmd, predicate::str::contains("test"));

    Ok(())
}

#[test]
fn two_new_notes() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("new", "test");

    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd, ["new", "second test"]);

    assert_success!(cmd, predicate::str::contains("Note with ID 1 created."));

    cmd = cmd_with_args!(cmd, ["get", "--all"]);

    assert_success!(
        cmd,
        predicate::str::contains("test").and(predicate::str::contains("second test"))
    );

    Ok(())
}

// Tests to add:
// - Adding a note.
// - Deleting a note.
// - Getting a note.
// - Editing a note.
// - Errors on incorrect usage.
