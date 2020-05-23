#![allow(dead_code)]

mod app;
mod errors;
mod notes;
mod table;
mod util;

use structopt::StructOpt;

use crate::app::{run_app, App};

fn main() -> anyhow::Result<()> {
    let app = App::from_args();

    run_app(app)?;

    Ok(())
}
