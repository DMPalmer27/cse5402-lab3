/* 
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: main.rs
 * Summary: This file contains helper functions to deal with the command line and the main
 * function which does the actual script running
 *
 */


pub mod lab3;

use lab3::return_wrapper::ReturnWrapper;
use lab3::server::Server;

use std::env;
use std::io::Write;

const EXPECTED_ARGS: usize = 2;
const PROG_NAME_INDEX: usize = 0;
const ADDR_INDEX: usize = 1;

const ERR_CMD_LINE: u8 = 1;
const ERR_CONNECTION: u8 = 2;

// This function is called whenever the program is ran with improper command line arguments and it
// prints a message telling the user how to run the program
fn usage(name: &String) {
    if let Err(_) = writeln!(std::io::stdout().lock(), "Usage: ./{name} <network_address>") {
        // Print fail
    }
}

// This function is used to parse the command line arguments. It ensures that the correct number of
// command line arguments were supplied and, if so, fills the address variable with the address
// that was passed in to the function.
fn parse_args(addr: &mut String) -> Result<(), u8> {
    let mut args = Vec::<String>::new();
    for arg in env::args() {
        args.push(arg);
    }

    if args.len() != EXPECTED_ARGS {
        usage(&args[PROG_NAME_INDEX]);
        return Err(ERR_CMD_LINE);
    }

    *addr = args[ADDR_INDEX].clone();
    Ok(())
}


// The main function executes the program which includes retrieving command line arguments,
// creating the server, and running the server.
fn main() -> ReturnWrapper {
    let mut address = String::new();

    if let Err(e) = parse_args(&mut address){
        return ReturnWrapper::new(Err(e));
    }

    let mut server = Server::new();
    server.open(&address);
    if !server.is_open() {
        if let Err(_) = writeln!(std::io::stderr().lock(), "Error: could not connect to the server with address {}", address){
            // Print fail
        }
        return ReturnWrapper::new(Err(ERR_CONNECTION));
    }
    server.run();
    ReturnWrapper::new(Ok(()))
}
