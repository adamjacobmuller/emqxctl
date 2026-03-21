use clap_complete::{generate, Shell};
use std::io;
use crate::cli::Cli;
use clap::CommandFactory;

pub fn execute(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "emqxctl", &mut io::stdout());
}
