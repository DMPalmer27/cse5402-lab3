/* 
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: main.rs
 * Summary: This file contains helper functions to deal with the command line and the
 * main function which does the actual play script creation 
 *
 */


pub mod lab3;

use std::env;
use std::io::Write;
use lab3::declarations;
use lab3::play::Play;
use lab3::return_wrapper::ReturnWrapper;


// This function is called whenver the program is ran with improper command line arguments and it
// prints a message telling the user how to run the program
fn usage(name: &String) {
    match writeln!(std::io::stdout().lock(), "Usage: ./{name} <script_file_name> [whinge]") {
        Ok(_) => {}, //success
        Err(_) => {}, //fail
    }
}

// This function is used to parse the command line arguments. It takes one parameter, a mutable
// reference to a string in which it places the name of the file provided as the first command line
// argument. It also sets the whinge mode flag if "whinge" was provided as the second command line
// argument. If the program was ran improperly it calls the usage function and returns an error.
fn parse_args(name: &mut String) -> Result<(), u8> {
    let mut args = Vec::<String>::new();
    for arg in env::args() {
        args.push(arg);
    }
    
    //Check if valid input
    if args.len() < declarations::MIN_ARGS  || 
    args.len() > declarations::MAX_ARGS || 
    (args.len() == declarations::MAX_ARGS && args[declarations::WHINGE_MODE] != "whinge".to_string()){

        usage(&args[declarations::PROG_NAME]);
        return Err(declarations::ERR_CMD_LINE);
    }

    *name = args[declarations::CONFIG_FILE].clone(); 
    
    if args.len() == declarations::MAX_ARGS {
        use std::sync::atomic::Ordering;
        declarations::WHINGE_ON.store(true, Ordering::SeqCst); 
    }
    Ok(())
}


// The main function executes the program which includes retrieving command line arguments,
// constructing the play, and printing the play.  
fn main() -> ReturnWrapper {
    let mut script_file: String = Default::default();

    if let Err(e) = parse_args(&mut script_file){
        return ReturnWrapper::new(Err(e));
    }

    let mut play = Play::new();
    if let Err(e) = play.prepare(&script_file){
        return ReturnWrapper::new(Err(e));
    }

    play.recite();
    
    ReturnWrapper::new(Ok(()))
}
