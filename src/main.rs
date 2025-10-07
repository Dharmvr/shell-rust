#[allow(unused_imports)]
use std::io::{self, Write};

use std::{fs, os::unix::fs::PermissionsExt};

#[derive(Debug, PartialEq)]
enum Command {
    LS { from: String, to: String },
    Echo { arg: String, files: String },
    Exit,
    Type(TypeCommand),
    External { program: String, args: Vec<String> },
    Unknown(String),
    PWD(String),
    CD(String),
    CAT(String),
    EXECCAT(String),
}
#[derive(Debug, PartialEq)]
enum TypeCommand {
    PWD,
    Echo,
    Exit,
    Type,
    External(String),
    CD,
}

fn handle_input(input: &str) -> (String, String, String) {
    let (args, file_part) = handle_redirect(input);
    let mut new_input = args.splitn(2, " ").map(str::trim);
    let command = new_input.next().unwrap_or("");
    let input = new_input.next().unwrap_or("");

    let input = handle_single_quote(&input);

    // println!("Command: {}", command);
    // println!("Input: {}", input);
    // println!("File Part: {}", file_part);

    // println!(
    //     "Debug: Processed input after handle_single_quote: {:?}",
    //     input
    // );
    // println!("{}",file_part);
    (command.to_string(), input.trim().to_string(), file_part)
}

fn input_parse(re_input: &str) -> Command {
    let i_vec = re_input.split_whitespace().collect::<Vec<&str>>();

    if i_vec.is_empty() {
        println!("");
        return Command::Unknown(re_input.trim().to_string());
    }
    let (command, input, _file_part) = handle_input(re_input);
    // let command = i_vec[0];
    let args = i_vec[1..].to_vec();
    if re_input.starts_with("'") || re_input.starts_with("\"") {
        return Command::EXECCAT(args[args.len() - 1].to_string());
    } else if command == "ls" {
        let from = input.clone();
        let to = _file_part.clone();
        return Command::LS { from, to };
    } else if command == "cat" {
        //   println!("{}", command);
        return Command::CAT(re_input.strip_prefix("cat").unwrap().to_string());
    } else if command == "cd" && args.len() == 1 {
        let path = args[0];
        return Command::CD(path.to_string());
    } else if command == "pwd" && args.len() == 0 {
        match std::env::current_dir() {
            Ok(path) => Command::PWD(path.display().to_string()),
            Err(_) => Command::Unknown(input.trim().to_string()),
        }
    } else if command == "echo" {
        return Command::Echo {
            arg: input,
            files: _file_part,
        };
    } else if command == "exit" && args.len() == 1 && args[0] == "0" {
        return Command::Exit;
    } else if command == "type" {
        if args.len() == 1 {
            let cmd = args[0];
            match cmd {
                // "cat" => return Command::Type(TypeCommand::CAT),
                "cd" => return Command::Type(TypeCommand::CD),
                "pwd" => return Command::Type(TypeCommand::PWD),
                "echo" => return Command::Type(TypeCommand::Echo),
                "exit" => return Command::Type(TypeCommand::Exit),
                "type" => return Command::Type(TypeCommand::Type),
                _ => return Command::Type(TypeCommand::External(cmd.to_string())),
            }
        } else {
            return Command::Unknown(input.trim().to_string());
        }
    } else if !command.is_empty() {
        return Command::External {
            program: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        };
    } else {
        return Command::Unknown(input.trim().to_string());
    }
}

fn handle_ls(from: &str, to: &str) {
    let mut from = from;
    let mut in_line = false;
    if from.starts_with("-1") {
        from = from.trim_start_matches("-1").trim();
        in_line = true;
    }
    let mut redirect_tot_err = false;
    if from.ends_with("2") {
        from = from.strip_suffix("2").unwrap();
        redirect_tot_err = true;
    }
    // println!("Debug: Argument to handle_ls - from: {:?}, to: {:?}", from, to);
    if from.is_empty() {
        from = ".";
    }

    let entries = fs::read_dir(from.trim());
    match entries {
        Ok(entries) => {
            let mut names = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        names.push(file_name);
                    }
                }
            }
            names.sort();
            let joined_names = names.join("\n");
            if to.is_empty() {
                for name in names.iter() {
                    if in_line {
                        println!("{}", name);
                    } else {
                        print!("{}  ", name);
                    }
                }

                println!();
            }
            if !to.is_empty() {
                match fs::write(to, &joined_names) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("ls: {}: No such file or directory", to);
                    }
                }
            }
        }
        Err(_) => {
            if redirect_tot_err {
                if !to.is_empty() {
                    let error = format!("ls: {}: No such file or directory", from.trim());
                    match fs::write(to, error) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("ls: {}: No such file or directory", to);
                        }
                    }
                }
            } else {
                println!("ls: cannot access '{}': No such file or directory", from);
            }
        }
    }
}

fn find_exec_function(path: &str) -> String {
    let dir_paths = std::env::var("PATH").unwrap_or_default();
    let dir_paths = dir_paths.split(':').collect::<Vec<&str>>();
    let res = String::new();
    for dir in dir_paths.iter() {
        let full_path = format!("{}/{}", dir, path);
        let new_path = std::path::Path::new(&full_path);

        if new_path.exists() {
            match fs::metadata(new_path) {
                Ok(metadata) => {
                    let permission = metadata.permissions();
                    let mode = permission.mode();
                    // Check if the file is executable by the user
                    if mode & 0o111 != 0 {
                        return new_path.display().to_string();
                    };
                }
                Err(_) => {}
            }
        }
    }

    res
}

fn handle_echo(arg: &str, files: &str) {
    // println!("Debug: Raw input to handle_echo: {:?}", input);
    // println!("Debug: Argument to handle_echo: {:?}", arg);
    let mut arg = arg;
    let mut redirect_err = false;
    // println!("{}",arg);
    if arg.ends_with("2") {
        arg = arg.strip_suffix("2").unwrap().trim();
        redirect_err = true
    }
    let input = &arg;
    // println!(
    //     "Debug: Processed input after handle_single_quote: {:?}",
    //     input
    // );
    // println!("{}", files);
    if files.is_empty() {
        println!("{}", input.trim());
    }
    let mut new_input = input;
    if !files.is_empty() {
        if redirect_err {
            new_input = &"";
        }

        match fs::write(&files, new_input.trim()) {
            Ok(_) => {
                if redirect_err {
                    println!("{}", input.trim());
                }
            }
            Err(_) => {
                if redirect_err {
                    println!("{}", input.trim());
                } else {
                    println!("echo: {}: No such file or directory", files);
                }
            }
        }
    }
}

fn handle_exit() {
    std::process::exit(0);
}

fn handle_unknown(input: &str) {
    println!("{}: command not found", input.trim());
}
fn handle_cd(path: &str) {
    let mut path = path.to_string();

    let home_dir = std::env::var("HOME").unwrap_or_default();
    if path.starts_with("~") {
        path = home_dir + &path[1..];
    }

    match std::env::set_current_dir(&path) {
        Ok(_) => {}
        Err(_) => {
            println!("cd: {}: No such file or directory", path);
        }
    }
}

fn handle_single_quote(input: &str) -> String {
    let mut result = String::new();
    let mut in_single_quote = false;
    let mut double_quote = false;
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch == '"' && !in_single_quote {
            double_quote = !double_quote;
            chars.next(); // Consume the quote
        } else if double_quote {
            if ch == '\\' {
                chars.next(); // Consume the backslash
                if let Some(&next_ch) = chars.peek() {
                    result.push(next_ch);
                    chars.next(); // Consume the escaped character
                    continue;
                }
            } else {
                result.push(ch);
                chars.next();
            }
            // Consume the character inside double quotes
        } else if ch == '\'' {
            in_single_quote = !in_single_quote;
            chars.next(); // Consume the quote
        } else if in_single_quote {
            result.push(ch);
            chars.next(); // Consume the character inside single quotes
        } else {
            if ch == ' ' && result.ends_with(' ') {
                chars.next(); // Skip extra spaces outside single quotes
                continue;
            }

            if ch == '\\' {
                chars.next(); // Consume the backslash
                if let Some(&next_ch) = chars.peek() {
                    result.push(next_ch);
                    chars.next(); // Consume the escaped character
                    continue;
                }
            }
            result.push(ch);
            chars.next(); // Consume the character outside single quotes
        }
    }

    result
}

fn handle_redirect(input: &str) -> (String, String) {
    // Split on '>' or on "1>" (as a substring)

    if let Some(idx) = input.find("1>") {
        let command_part = input[..idx].trim();
        let file_part = input[idx + 2..].trim();
        (command_part.to_string(), file_part.to_string())
    } else if let Some(idx) = input.find('>') {
        let command_part = input[..idx].trim();
        let file_part = input[idx + 1..].trim();
        (command_part.to_string(), file_part.to_string())
    } else {
        (input.trim().to_string(), "".to_string())
    }
}

fn handle_cat(args: String) {
    if args.is_empty() {
        println!("cat: missing file operand");
        return;
    }
    let mut redirect_to_err = false;
    if args.contains("2>") {
        redirect_to_err = true;
    }
    // println!("{}",args);
    let (mut args, file_part) = handle_redirect(&args);
    // let file_part ="";
    // println!("{}",args);
    // println!("{}", file_part);
    // println!("{}", args);

    if args.ends_with("2") {
        args = args.strip_suffix("2").unwrap().trim().to_string();
    }
    let mut new_files = Vec::new();
    let mut string_args = String::new();
    let mut single_quote = false;
    let mut double_quote = false;
    let mut err = false;
    for char in args.trim().chars() {
        // println!("single:{}", single_quote);
        // println!("double: {}", double_quote);
        if char == '"' && !single_quote {
            double_quote = !double_quote;

            continue;
        } else if double_quote {
            string_args.push(char);
            continue;
        } else if char == '\'' {
            single_quote = !single_quote;

            continue;
        }
        if char == ' ' && !single_quote && !double_quote {
            if !string_args.is_empty() {
                new_files.push(string_args.to_string());
                string_args = String::new();
            }
        } else {
            string_args.push(char);
        }
    }
    // println!("{}", single_quote);
    // println!("{}", double_quote);

    if !string_args.is_empty() {
        new_files.push(string_args.to_string());
    }
    // println!("{:?}", new_files);
    for file in new_files {
        //    println!("{}", file);
        match fs::read_to_string(&file) {
            Ok(contents) => {
                if file_part.is_empty() {
                    print!("{}", contents.trim());
                }

                if !file_part.is_empty() {
                    // println!("{}", contents.trim());
                    let new_cont = if redirect_to_err {
                        String::from("")
                    } else {
                        contents.clone()
                    };
                    match fs::write(&file_part, new_cont.trim()) {
                        Ok(_) => {
                            if redirect_to_err {
                                println!("{}",contents.trim());
                            }
                        }
                        Err(_) => {
                            println!("cat: {}: No such file or directory 1", file_part);
                            err = true;
                        }
                    }
                }
            }
            Err(_e) => {
                if redirect_to_err {
                    match fs::write(&file_part, format!("cat: {}: No such file or directory",file)){
                        Ok(_)=>{},
                        Err(_) => {
                            println!("cat: {}: No such file or directory 1", file_part);
                            err = true;
                        }
                    }
                } else {
                    println!("cat: {}: No such file or directory", file);
                    err = true;
                }
            }
        }
    }
    if err == false && file_part.is_empty() {
        println!();
    }
}

fn handle_exec_cat(args: String) {
    if args.is_empty() {
        println!("cat: missing file operand");
        return;
    }
    let file = fs::read_to_string(&args);
    match file {
        Ok(contents) => {
            print!("{}", contents);
        }
        Err(_) => {
            println!("cat: {}: No such file or directory", args);
        }
    }
}
fn main() {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();

        let mut input = String::new();

        stdin.read_line(&mut input).unwrap();
        match input_parse(&input) {
            Command::LS { from, to } => handle_ls(&from, &to),
            Command::EXECCAT(args) => handle_exec_cat(args),
            Command::CAT(args) => handle_cat(args),
            Command::CD(path) => handle_cd(&path),
            Command::PWD(path) => println!("{}", path),
            Command::Echo { arg, files } => handle_echo(&arg, &files),
            Command::Exit => handle_exit(),
            Command::Type(cmd) => match cmd {
                TypeCommand::CD => println!("cd is a shell builtin"),
                TypeCommand::PWD => println!("pwd is a shell builtin"),
                TypeCommand::Echo => println!("echo is a shell builtin"),
                TypeCommand::Exit => println!("exit is a shell builtin"),
                TypeCommand::Type => println!("type is a shell builtin"),
                TypeCommand::External(cmd) => {
                    let path = find_exec_function(&cmd);
                    if !path.is_empty() {
                        println!("{} is {}", cmd, path);
                    } else {
                        println!("{} not found", cmd);
                    }
                }
            },
            Command::External { program, args } => {
                let path = find_exec_function(&program);

                if !path.is_empty() {
                    match std::process::Command::new(&program).args(args).spawn() {
                        Ok(mut child) => {
                            child.wait().expect("Failed to wait on child process");
                        }
                        Err(_e) => {
                            println!("{}: command not found", &program);
                        }
                    }
                } else {
                    println!("{}: command not found", program);
                }
            }
            Command::Unknown(cmd) => handle_unknown(&cmd),
        }
    }
}
