use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use assert_cmd::prelude::*;
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

macro_rules! new_cmd {
    ($path:expr) => {{
        let mut cmd = Command::cargo_bin("notes")?;
        cmd.arg("--path").arg($path.as_os_str());

        cmd
    }};
}

struct TestCommand {
    pub cmd: Command,
    pub path: PathBuf,
    _tempdir: TempDir,
}

impl TestCommand {
    pub fn new() -> anyhow::Result<Self> {
        let tempdir = tempdir()?;
        let path = tempdir.path().to_owned().join("test");
        let cmd = new_cmd!(path);

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
        let mut new_cmd = new_cmd!($cmd.path);

        $(
            new_cmd.arg($arg);
        )*

        $cmd.cmd = new_cmd;

        $cmd
    }};
    ($cmd:ident) => {
        cmd_with_args!($cmd, [])
    };
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
    let mut cmd = cmd_with_args!("new", "first");
    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd, ["new", "second"]);
    assert_success!(cmd, predicate::str::contains("Note with ID 1 created."));

    cmd = cmd_with_args!(cmd, ["get", "--all"]);
    assert_success!(
        cmd,
        predicate::str::contains("first").and(predicate::str::contains("second"))
    );

    Ok(())
}

#[test]
fn get_specific_note() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("new", "first");
    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd, ["new", "second"]);
    assert_success!(cmd, predicate::str::contains("Note with ID 1 created."));

    cmd = cmd_with_args!(cmd, ["get", "0"]);
    assert_success!(cmd, predicate::str::contains("first"));

    cmd = cmd_with_args!(cmd, ["get", "1"]);
    assert_success!(cmd, predicate::str::contains("second"));

    Ok(())
}

#[test]
fn edit_note() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("new", "test");
    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd);
    assert_success!(cmd, predicate::str::contains("test"));

    cmd = cmd_with_args!(cmd, ["edit", "0", "--content", "\"other\""]);
    assert_success!(cmd, predicate::str::contains("Note 0 edited: \"other\""));

    cmd = cmd_with_args!(cmd);
    assert_success!(cmd, predicate::str::contains("other"));

    Ok(())
}

#[test]
fn delete_note() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("new", "test");
    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd);
    assert_success!(cmd, predicate::str::contains("test"));

    cmd = cmd_with_args!(cmd, ["delete", "0"]);
    cmd.cmd.write_stdin("y");
    assert_success!(cmd, predicate::str::contains("There are no notes."));
    Ok(())
}

#[test]
fn encrypt_notes() -> anyhow::Result<()> {
    let mut cmd = cmd_with_args!("new", "test");
    assert_success!(cmd, predicate::str::contains("Note with ID 0 created."));

    cmd = cmd_with_args!(cmd);
    assert_success!(cmd, predicate::str::contains("test"));

    cmd = cmd_with_args!(cmd, ["--encrypt"]);
    assert_success!(cmd, predicate::str::contains("Notes file encrypted."));

    cmd = cmd_with_args!(cmd);
    cmd.cmd.write_stdin("y");
    assert_success!(cmd, predicate::str::contains("test"));
    Ok(())
}

// Tests to add:
// - Deleting a note.
// - Getting a note.
// - Editing a note.
// - Errors on incorrect usage.
