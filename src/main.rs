#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command as ProcessCommand;
use std::{fs, os::unix::fs::PermissionsExt};

#[derive(Debug, PartialEq)]
enum Command {
    Echo(String),
    Exit,
    Type(TypeCommand),
    External { program: String, args: Vec<String> },
    Unknown(String),
    PWD(String),
    CD(String),
    CAT(String),
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

fn input_parse(input: &str) -> Command {
    let i_vec = input.split_whitespace().collect::<Vec<&str>>();

    if i_vec.is_empty() {
        println!("");
        return Command::Unknown(input.trim().to_string());
    }
    let command = i_vec[0];
    let args = i_vec[1..].to_vec();

    if command == "cat" {
        //   println!("{}", command);
        return Command::CAT(input.strip_prefix("cat").unwrap().to_string());
    } else if command == "cd" && args.len() == 1 {
        let path = args[0];
        return Command::CD(path.to_string());
    } else if command == "pwd" && args.len() == 0 {
        match std::env::current_dir() {
            Ok(path) => Command::PWD(path.display().to_string()),
            Err(_) => Command::Unknown(input.trim().to_string()),
        }
    } else if command == "echo" {
        return Command::Echo(input.strip_prefix("echo").unwrap().to_string());
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

fn handle_echo(input: &str) {
    // println!("Debug: Raw input to handle_echo: {:?}", input);
    let input = handle_single_quote(input);
    // println!(
    //     "Debug: Processed input after handle_single_quote: {:?}",
    //     input
    // );
    println!("{}", input.trim());
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
            result.push(ch);
            chars.next();
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

            if ch=='\\'{
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

fn handle_cat(args: String) {
    if args.is_empty() {
        println!("cat: missing file operand");
        return;
    }
    // println!("{}", args);
    let mut new_files = Vec::new();
    let mut string_args = String::new();
    let mut single_quote = false;
    let mut double_quote = false;
    for char in args.chars() {
        if char == '"' {
            double_quote = !double_quote;

            continue;
        } else if double_quote {
            string_args.push(char);
            continue;
        } else if char == '\'' {
            single_quote = !single_quote;

            continue;
        }
        if char == ' ' && !single_quote {
            if !string_args.is_empty() {
                new_files.push(string_args.to_string());
                string_args = String::new();
            }
        } else {
            string_args.push(char);
        }
    }
    // println!("{}", string_args);
    if !string_args.is_empty() {
        new_files.push(string_args.strip_suffix("\n").unwrap().to_string());
    }
    // println!("{:?}", new_files);
    for file in new_files {
        //    println!("{}", file);
        match fs::read_to_string(&file) {
            Ok(contents) => {
                print!("{}", contents);
            }
            Err(_) => {
                println!("cat: {}: No such file or directory", file);
            }
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
            Command::CAT(args) => handle_cat(args),
            Command::CD(path) => handle_cd(&path),
            Command::PWD(path) => println!("{}", path),
            Command::Echo(args) => handle_echo(&args),
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
