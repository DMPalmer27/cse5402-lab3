/*
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: return_wrapper.rs
 * Summary: This file declares and implements the ReturnWrapper struct which is 
 * used by the main function to return custom exit codes for different types of
 * failure.
 */


use std::process::{Termination, ExitCode};
use std::io::Write;

const SUCCESS: u8 = 0;

pub struct ReturnWrapper {
    val: u8,
}

impl ReturnWrapper {
    pub fn new(r: Result<(), u8>) -> Self {
        match r {
            Ok(_) => Self { val: SUCCESS },
            Err(e) => Self { val: e },
        }
    }
}

impl Termination for ReturnWrapper {
    fn report(self) -> ExitCode {
        if self.val != SUCCESS {
            match writeln!(std::io::stderr().lock(), "Error: {}", self.val) {
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
        }
        ExitCode::from(self.val)
    }
}
