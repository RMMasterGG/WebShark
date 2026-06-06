use std::collections::HashMap;
use std::env;
use std::fmt::format;
use std::sync::Arc;

type CommandHandler = Arc<dyn Fn(Vec<String>) + Send + Sync>;

pub struct Command {
    name: &'static str,
    usages: Vec<String>,
    aliases: Vec<&'static str>,
    description: &'static str,
    subcommands: HashMap<String, Command>,
    handler: CommandHandler,
}

impl Command {
    pub fn new<F>(name: &'static str, description: &'static str, handler: F) -> Self
    where
        F: Fn(Vec<String>) + Send + Sync + 'static,
    {
        Self {
            name,
            usages: Vec::new(),
            aliases: Vec::new(),
            description,
            subcommands: HashMap::new(),
            handler: Arc::new(handler),
        }
    }

    pub fn add_alias(mut self, alias: &'static str) -> Self {
        self.aliases.push(alias);
        self
    }

    pub fn add_subcommand(mut self, subcommand: Command) -> Self {
        self.subcommands.insert(subcommand.name.to_owned(), subcommand);
        self
    }

    pub fn add_usage(mut self, usage: &'static str) -> Self {
        self.usages.push(usage.to_owned());
        self
    }

    pub fn find_subcommand(&self, name: &str) -> Option<&Command> {
        self.subcommands.get(name).or_else(|| {
            self.subcommands.values().find(|cmd| cmd.aliases.contains(&name))
        })
    }

    pub fn invoke(&self, mut args: Vec<String>) {
        if !args.is_empty() {

            let next_possible_subcommand = &args[0];

            if let Some(subcommand) = self.find_subcommand(next_possible_subcommand) {
                args.remove(0);
                subcommand.invoke(args);
                return;
            }
        }
        (self.handler)(args);
    }
}

pub struct Cli {
    name: &'static str,
    commands: HashMap<String, Command>,
}

impl Cli {
    pub fn new(name: &'static str) -> Self {
        Self { name, commands: HashMap::new() }
    }

    pub fn add_command(mut self, command: Command) -> Self {
        self.commands.insert(command.name.to_owned(), command);
        self
    }

    fn print_help(&self) {
        let mut max_width = 0;
        for (name, cmd) in &self.commands {
            let command_alias = format!("{} ({})", name, cmd.aliases.join(", "));

            let full_command = format!("{} {}", command_alias, cmd.usages.join(", "));

            if full_command.len() > max_width {
                max_width = full_command.len();
            }
        }

        let width = max_width + 2;

        println!("Доступные команды для {}:", self.name);
        for (name, cmd) in &self.commands {

            let aliases = cmd.aliases.join(", ");

            let mut command_alias = String::with_capacity(name.len() + aliases.len());

            command_alias.push_str(name);
            command_alias.push_str( " (");
            command_alias.push_str(&aliases);
            command_alias.push_str( ") ");

            let usages = cmd.usages.join(", ");

            let mut full_command = String::with_capacity(command_alias.len() + usages.len());

            full_command.push_str(&command_alias);
            full_command.push_str(&usages);

            println!("  {:<width$} {}", full_command, cmd.description);
        }
    }

    pub fn run(&self) {
        let args = env::args().collect::<Vec<String>>();

        if args.len() < 2 || &args[1] == "help" {
            self.print_help();
            return;
        }

        let target_command = &args[1];

        let sub_args = args[2..].to_vec();

        let found_command = self.commands.get(target_command)
            .or_else(|| self.commands.values().find(|cmd| cmd.aliases.contains(&target_command.as_str())));

        match found_command {
            Some(cmd) => cmd.invoke(sub_args),
            None => println!("Unknown command: {}", target_command),
        }

    }
}