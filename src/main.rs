#![allow(dead_code)]

mod app;
mod crypto;
mod display;
mod errors;
mod notes;
mod tags;
mod util;

use crate::app::{app, run_app};

fn main() -> anyhow::Result<()> {
    let app = app();

    run_app(app)?;

    Ok(())
}
