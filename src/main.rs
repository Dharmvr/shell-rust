#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command as ProcessCommand;
use std::{fs, os::unix::fs::PermissionsExt};

#[derive(Debug, PartialEq)]
enum Command {
    Echo(Vec<String>),
    Exit,
    Type(TypeCommand),
    External { program: String, args: Vec<String> },
    Unknown(String),
    PWD(String)
}
#[derive(Debug, PartialEq)]
enum TypeCommand {
    PWD,
    Echo,
    Exit,
    Type,
    External(String),
}

fn input_parse(input: &str) -> Command {
    let i_vec = input.split_whitespace().collect::<Vec<&str>>();
    if i_vec.is_empty() {
        println!("");
        return Command::Unknown(input.trim().to_string());
    }
    let command = i_vec[0];
    let args = i_vec[1..].to_vec();

    if command == "pwd" && args.is_empty() {
        match std::env::current_dir() {
            Ok(path) => {
                return Command::PWD(path.display().to_string());
            }
            Err(_) => {
                return Command::Unknown(input.trim().to_string());
            }
        }
    } else if command == "echo" {
        return Command::Echo(args.iter().map(|s| s.to_string()).collect());
    } else if command == "exit" && args.len() == 1 && args[0] == "0" {
        return Command::Exit;
    } else if command == "type" {
        if args.len() == 1 {
            let cmd = args[0];
            match cmd {
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
    println!("{}", input.trim());
}

fn handle_exit() {
    std::process::exit(0);
}

fn handle_unknown(input: &str) {
    println!("{}: command not found", input.trim());
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
            Command::PWD(path) => println!("{}", path),
            Command::Echo(args) => handle_echo(&args.join(" ")),
            Command::Exit => handle_exit(),
            Command::Type(cmd) => match cmd {
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
