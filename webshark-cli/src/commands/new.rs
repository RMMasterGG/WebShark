pub fn command_new(args: Vec<String>) {
    if args.is_empty() || args.len() > 2 {
        println!("Not enough arguments");
        return;
    }

    let name = args[0].clone();

    print!("Create new project: {}", name);
}
