pub mod command_handler;
pub mod commands;

use crate::command_handler::{Cli, Command};
use crate::commands::new::command_new;
use crate::commands::generate::command_generate;
use crate::commands::generate::controller::command_controller;

fn main() {

    let command_new = Command::new("new", "Create new project", command_new)
        .add_usage("<PROJECT_NAME>")
        .add_alias("n");

    let command_generate = Command::new("generate", "Generate modules for your app", command_generate)
        .add_usage("<TYPE>")
        .add_usage("<NAME>")
        .add_alias("g")
        .add_subcommand(Command::new("controller", "Create component", command_controller).add_alias("c"));

    let app = Cli::new("webshark")
        .add_command(command_new)
        .add_command(command_generate);

    app.run();
}