#[allow(unused_imports)]
use std::io::{self, Write};
enum Command {
    Echo,
    Exit,
    Unknown,
}
impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Echo => write!(f, "echo"),
            Command::Exit => write!(f, "exit"),
            Command::Unknown => write!(f, "unknown"),
        }
    }
}

type CommandHandler = fn(&str);

fn handle_echo(input: &str) {
    let res = input.strip_prefix("echo").unwrap();
    println!("{}", res);
}

fn handle_exit(_input: &str) {
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
        let input = input.trim();
        if input.starts_with("type"){
            let command=  input.strip_prefix("type").unwrap().trim();
            match command {
                "type" => println!("type is a shell builtin"),
                "echo" => println!("echo is a shell builtin"),
                "exit" => println!("exit is a shell builtin"),
                _ => println!("{}: not found", command)
                
            }

        }
        else if input.starts_with("echo") {
            let res = &input.strip_prefix("echo").unwrap();

            println!("{}", res.trim());
        } else if input.trim() == "exit 0" {
            return;
        } else {
            println!("{}: command not found", input.trim())
        }
    }
}
