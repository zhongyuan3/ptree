mod color;
mod command;
mod config;
mod error;
mod filter;
mod format;
mod icon;

use std::process::exit;

use clap::Parser;

use crate::command::CmdArgs;
use crate::command::exec_cmd;

fn main() {
    if let Err(e) = exec_cmd(&CmdArgs::parse()) {
        eprintln!("{}", e);
        exit(1);
    }
}
