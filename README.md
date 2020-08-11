Notes
------------
[![ci](https://github.com/alidiusk/notes-rs/workflows/ci/badge.svg)](https://github.com/alidiusk/notes-rs/actions)
[![crates.io](https://img.shields.io/crates/v/notes-rs.svg)](https://crates.io/crates/notes-rs)

Notes is a simple command line notes application for storing quick, short notes.

Licensed under GPLv3.

### Installation
#### From Binaries
There are binaries for Linux and macOS on the [Releases page](https://github.com/alidiusk/notes-rs/releases).

#### From source
`Notes` can be built from source and installed with `cargo`.
```bash
cargo install notes-rs
```

### Building
`Notes` can be built from source on the `stable`, `beta`, and `nightly` compilers. You must have a rust compiler and `cargo` installed.
```bash
$ git clone https://github.com/alidiusk/notes-rs
$ cd notes-rs
$ cargo build --release
$ ./target/release/notes --version
0.2.1
```

### Example Usage

```bash
# prints all notes (without descriptions). there are no notes yet.
$ notes

# creates a new note with the given content, no tags, no description.
$ notes new "Learn to use notes-rs."

$ notes new "This note has tags and a description." --tags learning --desc "test note."

# get all notes and display their descriptions
$ notes get --desc

# get all notes with the `learning` tag
$ notes get --tags learning

# change the tag on note 1
$ notes edit 1 --tags "new-tag"

# delete note 0; there will be a confirmation prompt displaying its content.
$ notes delete 0
```

### Features

* Notes consisting of an id, time, tags, content, and description.
* Creation of notes on the command line, through an editor, or from a file.
* Optional tags to add context to notes.
* Optional description to provide further information for a note.
* Automatically managed note creation / edit times.
* Modifications of a note's content, tags, or description.
* Retrieval of all notes, a specific note, or all notes with a given tag(s).
* Deletion of a note given its id.
* Custom notes path with the `--path` option (default is XDG data directory).
