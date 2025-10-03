#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Uncomment this block to pass the first stage
    loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    
    let mut input = String::new();
     
    
    stdin.read_line(&mut input).unwrap();
    let  input = input.trim(); 
    if input.starts_with("echo") {
        let res =&input.strip_prefix("echo").unwrap();

        println!("{}",res.trim() );
    }
   
    else if input.trim() =="exit 0" {
        return;
    }
    else {
        println!("{}: command not found",input.trim())
    }
    
   
    }
}
