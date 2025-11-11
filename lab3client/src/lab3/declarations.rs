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

pub const ERR_CMD_LINE: u8= 1;
pub const ERR_SCRIPT_GEN: u8= 2;

use std::sync::atomic::AtomicBool;
pub static WHINGE_ON: AtomicBool = AtomicBool::new(false);


use std::fs::File;
use std::io::{BufReader, BufRead};

// This function is used to open and read lines from a file. 
// Ita Result type that is an error if a file could not be opened or read from,
// and success otherwise.
pub fn grab_trimmed_file_lines(file_name: &str, file_lines: &mut Vec<String>) -> Result<(), u8> {
    match File::open(file_name) {
        Err(_) => {
            eprintln!("Error: script generation failed because the file {} could not be opened", file_name);
            return Err(ERR_SCRIPT_GEN);
        },
        Ok(f) => {
            let mut reader = BufReader::new(f);
            let mut s = String::new();
            loop {
                s.clear();
                match reader.read_line(&mut s) {
                    Err(_) => {
                        eprintln!("Error: script generation failed because line could not be read");
                        return Err(ERR_SCRIPT_GEN);
                    },
                    Ok(bytes_read) => {
                        if bytes_read == 0 { //done reading
                            return Ok(())
                        }
                        file_lines.push(s.trim().to_string());
                    },
                }

            }
        },
    }
}
