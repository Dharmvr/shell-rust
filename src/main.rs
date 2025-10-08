use std::fs::OpenOptions;
use std::io::stdout;
#[allow(unused_imports)]
use std::io::{self, Write};

use std::process::{self, Command};
use std::str::FromStr;
use std::{fs, os::unix::fs::PermissionsExt, string};

#[derive(Debug)]
enum RedirectType {
    Stdout,
    Stderr,
    Stdin,
}

#[derive(Debug)]
struct Redirect {
    redirect_type: RedirectType,
    target: String,
    append: bool,
}
#[derive(Debug, PartialEq)]

enum ShellCommand {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Unknown,
}

impl FromStr for ShellCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exit" => Ok(ShellCommand::Exit),
            "echo" => Ok(ShellCommand::Echo),
            "type" => Ok(ShellCommand::Type),
            "pwd" => Ok(ShellCommand::Pwd),
            "cd" => Ok(ShellCommand::Cd),
            _ => Ok(ShellCommand::Unknown),
        }
    }
}
//
fn parse(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            '\\' => {
                if let Some(next) = chars.next() {
                    if in_single {
                        // Backslash is literal in single quotes
                        current.push('\\');
                        current.push(next);
                    } else if in_double {
                        // In double quotes, only " and \ are escaped
                        if next == '"' || next == '\\' {
                            current.push(next);
                        } else {
                            // keep backslash literal for other characters
                            current.push('\\');
                            current.push(next);
                        }
                    } else {
                        // Outside quotes, backslash escapes next char
                        current.push(next);
                    }
                }
            }
            ' ' if !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        args.push(current.trim().to_string());
    }

    args
}

fn handle_args(args: Vec<String>) -> (Vec<String>, Vec<Redirect>) {
    let mut final_args = Vec::new();
    let mut redirects = Vec::new();

    let mut iter = args.into_iter().peekable();
    while let Some(token) = iter.next() {
        match token.as_str() {
            ">" | "1>" | ">>" | "1>>" | "2>" | "2>>" => {
                if let Some(path) = iter.next() {
                    let redirect_type = if token.starts_with("2") {
                        RedirectType::Stderr
                    } else {
                        RedirectType::Stdout
                    };
                    let append = token.ends_with(">>");
                    redirects.push(Redirect {
                        redirect_type,
                        target: path,
                        append,
                    });
                } else {
                    eprintln!("syntax error near unexpected token `newline`");
                }
            }
            "<" => {
                if let Some(path) = iter.next() {
                    redirects.push(Redirect {
                        redirect_type: RedirectType::Stdin,
                        target: path,
                        append: false,
                    });
                } else {
                    eprintln!("syntax error near unexpected token `newline`");
                }
            }
            _ => final_args.push(token),
        }
    }

    (final_args, redirects)
}

fn main() {
    loop {
        print!("$ ");
        stdout().flush().unwrap();

        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // print!("{}", input);

        let mut args = parse(&input);
        // println!("{:?}", args);
        let command = args.remove(0);
        let mut new_command = &command;

        // println!("{}", command);
        let (new_args, redir) = handle_args(args.clone());
        // println!("{:?}", new_args);
        // println!("{:?}", re);

        let command = command.parse::<ShellCommand>().unwrap();
        match command {
            ShellCommand::Echo => {
                handle_echo(new_args, redir);
            }
            ShellCommand::Exit => {
                handle_exit();
            }
            ShellCommand::Pwd => handle_pwd(),
            ShellCommand::Cd => handle_cd(new_args),
            ShellCommand::Type => handle_type(new_args),
            ShellCommand::Unknown => {
                // let command = args.remove(0);
                if input.starts_with("\"") {
                    
                }
                handle_unknown(new_command.to_string(), new_args, redir)
            }
        }
    }
}

use std::path::Path;

fn open_file_for_redirect(target: &str, append: bool) -> std::fs::File {
    let path = Path::new(target);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("cannot create directory {}: {}", parent.display(), e);
                std::process::exit(1);
            });
        }
    }

    let file = if append {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap_or_else(|e| {
                eprintln!("cannot open {}: {}", path.display(), e);
                std::process::exit(1);
            })
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap_or_else(|e| {
                eprintln!("cannot open {}: {}", path.display(), e);
                std::process::exit(1);
            })
    };

    file
}


fn handle_echo(args: Vec<String>, redir: Vec<Redirect>) {
    let s = args.join(" ");

    // No redirection: print to terminal
    if redir.is_empty() {
        println!("{}", s);
        return;
    }

    for r in redir {
    

        match r.redirect_type {
            RedirectType::Stdout => {
                 let mut file = open_file_for_redirect(&r.target, r.append);
                // always add newline when echoing
                writeln!(file, "{}", s).unwrap_or_else(|e| {
                    eprintln!("echo: cannot write to {}: {}", r.target, e);
                    std::process::exit(1);
                });
            }

            RedirectType::Stderr => {
                 let mut file = open_file_for_redirect(&r.target, r.append);
                write!(file, "{}", "").unwrap_or_else(|e| {
                    eprintln!("echo: cannot write to {}: {}", r.target, e);
                    std::process::exit(1);
                });
               println!("{}",s);
            }

            RedirectType::Stdin => {
                // ignore stdin for echo
            }
        }
    }
}

fn handle_unknown(command: String, args: Vec<String>, redir: Vec<Redirect>) {
    let exec_path = find_exec_function(&command);

    if exec_path.is_empty() {
        eprintln!("{}: command not found", command);
        return;
    }
    
    let mut cmd = std::process::Command::new(exec_path);
    // println!("{:?}",cmd);
    cmd.args(args);

    for r in redir {
        match r.redirect_type {
            RedirectType::Stdout => {
                if r.append {
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&r.target)
                        .unwrap_or_else(|e| {
                            eprintln!("{}: cannot append to {}: {}", command, r.target, e);
                            std::process::exit(1);
                        });

                    // Ensure file ends with newline before appending command output
                    if let Ok(content) = std::fs::read_to_string(&r.target) {
                        if !content.is_empty() && !content.ends_with('\n') {
                            writeln!(file).unwrap();
                        }
                    }

                    cmd.stdout(file);
                } else {
                    let file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&r.target)
                        .unwrap_or_else(|e| {
                            eprintln!("{}: cannot write to {}: {}", command, r.target, e);
                            std::process::exit(1);
                        });
                    cmd.stdout(file);
                }
            }

            RedirectType::Stderr => {
                let file = if r.append {
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&r.target)
                        .unwrap()
                } else {
                    OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&r.target)
                        .unwrap()
                };
                cmd.stderr(file);
            }
            RedirectType::Stdin => {
                let file = OpenOptions::new().read(true).open(&r.target).unwrap();
                cmd.stdin(file);
            }
        }
    }

    match cmd.spawn() {
        Ok(mut child) => {
            child.wait().expect("Failed to wait on child process");
        }
        Err(e) => {
            eprintln!("{}: failed to execute: {}", command, e);
        }
    }
}

fn handle_exit() {
    process::exit(0);
}

fn handle_pwd() {
    match std::env::current_dir() {
        Ok(path) => {
            println!("{}", path.display().to_string())
        }
        Err(_) => {}
    };
}

fn handle_cd(args: Vec<String>) {
    let path = &args[0];

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

fn handle_type(args: Vec<String>) {
    let path = &args[0];
    let current_path = path.parse::<ShellCommand>().unwrap();
    if current_path != ShellCommand::Unknown {
        println!("{} is a shell builtin", path);
    } else {
        let result = handle_exec_function(path);
        if !result.is_empty() {
            println!("{} is {}", path, result);
        } else {
            println!("{} not found", path);
        }
    }
}
fn handle_exec_function(path: &str) -> String {
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
                        return path.to_string();
                    };
                }
                Err(_) => {}
            }
        }
    }

    res
}


