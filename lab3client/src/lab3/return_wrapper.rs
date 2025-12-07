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

// This block contains the implementation for a ReturnWrapper which is solely the constructor
impl ReturnWrapper {
    pub fn new(r: Result<(), u8>) -> Self {
        match r {
            Ok(_) => Self { val: SUCCESS },
            Err(e) => Self { val: e },
        }
    }
}

// This block contains the implementation of the Termination trait for ReturnWrapper which allows
// it to specify the values returned by the main program. It allows the program to specify its
// termination behavior.
impl Termination for ReturnWrapper {
    fn report(self) -> ExitCode {
        if self.val != SUCCESS {
            if let Err(_) = writeln!(std::io::stderr().lock(), "Error: {}", self.val) {
                // Print fail
            }
        }
        ExitCode::from(self.val)
    }
}
