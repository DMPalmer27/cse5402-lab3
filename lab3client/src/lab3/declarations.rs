/*
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: declarations.rs
 * Summary: This file contains global constants and utilities that multiple other
 * files in the program use.
 *
 */


pub const MIN_ARGS: usize = 2;
pub const MAX_ARGS: usize = 3;
pub const PROG_NAME: usize = 0;
pub const CONFIG_FILE: usize = 1;
pub const WHINGE_MODE: usize = 2;

pub const ERR_CMD_LINE: u8 = 1;
pub const ERR_SCRIPT_GEN: u8 = 2;
pub const ERR_MUTEX: u8 = 3;

const NETWORK_PREFIX: &str = "net:";
const NETWORK_IP_INDEX: usize = 0;
const NETWORK_PORT_INDEX: usize= 1;
const NETWORK_FILENAME_INDEX: usize = 2;
const EXPECTED_NETWORK_PARTS: usize = 3;
const ERR_NETWORK_FILENAME: u8 = 4;
const ERR_NETWORK_CONNECT: u8 = 5;

use std::sync::atomic::AtomicBool;
pub static WHINGE_ON: AtomicBool = AtomicBool::new(false);


use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::net::TcpStream;

// This function is used to open and read lines from a file. 
// Ita Result type that is an error if a file could not be opened or read from,
// and success otherwise.
pub fn grab_trimmed_file_lines(file_name: &str, file_lines: &mut Vec<String>) -> Result<(), u8> {
    match get_buffered_reader(file_name) {
        Err(e) => Err(e),
        Ok(mut reader) => {
            let mut s = String::new();
            loop {
                s.clear();
                match reader.read_line(&mut s) {
                    Err(_) => {
                        if let Err(_) = writeln!(std::io::stderr().lock(), "Error: script generation failed because line could not be read"){
                            //Print fail
                        }
                        return Err(ERR_SCRIPT_GEN);
                    }
                    Ok(bytes_read) => {
                        if bytes_read == 0 { //done reading
                            return Ok(());
                        }
                        file_lines.push(s.trim().to_string());
                    }
                }
            }
        }
    }
}

//This function takes a line which is either the name of a text file or a file that exists over the
//network. It detects the difference, in either case attempts to open the file or get the lines
//from the network, and returns a BufReader holding these lines
fn get_buffered_reader(line: &str) -> Result<BufReader<Box<dyn std::io::Read>>, u8> {
    if let Some(rest) = line.strip_prefix(NETWORK_PREFIX) {
        // Split remaining string at ":" to isolate the IP, port, and filename
        let parts: Vec<&str> = rest.splitn(3, ":").collect();
        if parts.len() != EXPECTED_NETWORK_PARTS {
            if let Err(_) = writeln!(std::io::stderr().lock(), "Error: line {} has a network prefix but not 3 distinct IP, port, and file name separated by colons after", line){
                //Print fail
            }
            return Err(ERR_NETWORK_FILENAME);
        }

        let ip_port = format!("{}:{}", parts[NETWORK_IP_INDEX], parts[NETWORK_PORT_INDEX]);
        let filename = parts[NETWORK_FILENAME_INDEX];

        match TcpStream::connect(&ip_port){
            Err(_) => {
                if let Err(_) = writeln!(std::io::stderr().lock(), "Error: failed to connect to address {ip_port}"){
                    // Print fail
                }
                Err(ERR_NETWORK_CONNECT)
            }
            Ok(mut stream) => {
                if let Err(_) = writeln!(stream, "{}", filename){
                    if let Err(_) = writeln!(std::io::stderr().lock(), "Error: write to stream fail"){
                        // Print fail
                    }
                }
                Ok(BufReader::new(Box::new(stream)))
            }
        }
    } else {
        match File::open(line) {
            Err(_) => {
                if let Err(_) = writeln!(std::io::stderr().lock(), "Error: script generation failed because the file {} could not be opened", line){
                    //Print fail
                }
                Err(ERR_SCRIPT_GEN)
            }
            Ok(file) => {
                Ok(BufReader::new(Box::new(file)))
            }
        }
    }
}
