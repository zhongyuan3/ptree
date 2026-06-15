mod color;
mod command;
mod config;
mod error;
mod filter;
mod format;
mod icon;
mod settings;
mod sort;

use std::process::exit;

use clap::Parser;

use crate::command::{App, AppCommand, exec_cmd, generate_completions};

fn main() {
    let app = App::parse();
    let result = match app.command {
        Some(AppCommand::Completions { shell }) => {
            generate_completions(shell);
            Ok(())
        }
        None => exec_cmd(&app.args),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        exit(1);
    }
}
