/* 
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: main.rs
 * Summary: This file contains a client program that is used to test the server. It very simply
 * just triggers the server to print the contents of a file over a connection, and quit if "quit"
 * was received.
 *
 */

use std::env;
use std::io::{Write, BufRead, BufReader};
use std::net::TcpStream;

const EXPECTED_ARGS: usize = 3;
const PROG_NAME_INDEX: usize = 0;
const ADDR_INDEX: usize = 1;
const TOKEN_INDEX: usize = 2;
const QUIT_STR: &str = "quit";

const ERR_CMD_LINE: u8 = 1;
const ERR_TCP_CONNECT: u8 = 2;


// This function is called whenever the program is ran with improper command line arguments and it
// prints a message telling the user how to run the program
fn usage(name: &String) {
    println!("Usage: ./{name} <network_address> <token>");
}


// This function parses the command line arguments, ensuring that the proper amount of arguments
// have been supplied. If they have, it properly fills the address and token variables so that they
// are ready for use by the main function.
fn parse_args(addr: &mut String, token: &mut String) -> Result<(), u8> {
    let mut args = Vec::<String>::new();
    for arg in env::args() {
        args.push(arg);
    }

    if args.len() != EXPECTED_ARGS {
        usage(&args[PROG_NAME_INDEX]);
        return Err(ERR_CMD_LINE);
    }

    *addr = args[ADDR_INDEX].clone();
    *token = args[TOKEN_INDEX].clone();
    Ok(())
}


// This is the main function that actually runs the server. 
fn main() -> Result<(), u8>{
    let mut address = String::new();
    let mut token = String::new();

    if let Err(e) = parse_args(&mut address, &mut token) {
        return Err(e);
    }

    match TcpStream::connect(&address) {
        Err(_) => {
            eprintln!("Error: failed to connect to address {address}");
            return Err(ERR_TCP_CONNECT);
        }
        Ok(mut stream) => {
            if let Err(_) = writeln!(stream, "{}", token) {
                eprintln!("Error: write to stream fail");
            }

            if token == QUIT_STR {
                let delay = std::time::Duration::from_secs(1);
                std::thread::sleep(delay);
                if let Err(_) = TcpStream::connect(&address) {
                    eprintln!("Error: failed to connect to address {address} after quit was given");
                }
                return Ok(());
            }

            let reader = BufReader::new(stream);
            for try_line in reader.lines() {
                match try_line {
                    Ok(line) => println!("{}", line),
                    Err(_) => {
                        eprintln!("Error reading line from server");
                        break;
                    }
                }
            }

        }
    }
    Ok(())
}
