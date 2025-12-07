/* 
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: server.rs
 * Summary: This file contains the Server struct and its implementation. It contains all the
 * utilities for creating, opening, and running the server.
 *
 */

use std::sync::atomic::{Ordering, AtomicBool};
use std::net::TcpListener;
use std::thread;
use std::io::{BufReader, BufRead, Write};
use std::fs::File;

static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);


const QUIT_STR: &str = "quit";


pub struct Server {
    listener: Option<TcpListener>,
    listening_addr: String,
}

// This implementation block includes all functionality for the server
impl Server {
    pub fn new() -> Self {
        Self {
            listener: None,
            listening_addr: "".to_string(),
        }
    }

    //This method checks whether the server is open, returning a boolean
    pub fn is_open(&self) -> bool {
        if let None = self.listener {
            false
        } else {
            true
        }
    }

    //This method opens the server using the address that is passed in to it
    pub fn open(&mut self, addr: &str) {
        if let Ok(lstnr) = TcpListener::bind(addr) {
            self.listener = Some(lstnr);
            self.listening_addr = addr.to_string();
        }
    }


    //This method runs the server. If the cancel flag is false and the listener exists, it
    //repeatedly accepts connections, reads in a token from the client, if the token is quit it
    //sets the cancel flag so that the loop is exited and otherwise treats the token as the name of
    //a file which it opens and sends each line over the socket. Each accepted client is ran within
    //its own thread.
    pub fn run(&self) {
        let mut thread_handles = Vec::new();
        while !CANCEL_FLAG.load(Ordering::SeqCst){
            if let Some(ref lstnr) = self.listener {
                match lstnr.accept() {
                    Err(_) => {
                        if let Err(_) = writeln!(std::io::stderr().lock(), "connection could not be accepted"){
                            // Print fail
                        }
                    }
                    Ok((mut socket, _addr)) => {
                        if CANCEL_FLAG.load(Ordering::SeqCst) { break; }
                        let handle = thread::spawn(move || {
                            let mut reader = BufReader::new(&mut socket);
                            let mut token = String::new();
                            if let Err(_) = reader.read_line(&mut token) {
                                // Panic becuase we are inside a thread so it will propagate an Err
                                // to the parent
                                panic!("Read failed in thread");
                            }

                            token = token.trim().to_string();
                            
                            if token == QUIT_STR {
                                CANCEL_FLAG.store(true, Ordering::SeqCst);
                                return;
                            }

                            if token.contains("/")
                                || token.contains("\\")
                                || token.contains("..")
                                || token.contains("$") {
                                //Token indicates a path, don't allow it
                                if let Err(_) = writeln!(std::io::stderr().lock(), "Token {} that was received by the server indicates a path and is hence not allowed", token) {
                                    // Print fail
                                }
                                return;
                            }

                            let f = match File::open(token) {
                                Ok(f) => f,
                                Err(_) => {
                                    if let Err(_) = writeln!(std::io::stderr().lock(), "File could not be opened"){
                                        // Print fail
                                    }
                                    return;
                                }
                            };

                            //f is a valid open file
                            let f_reader = BufReader::new(f);
                            for try_line in f_reader.lines() {
                                if let Ok(line) = try_line {
                                    // write line over eonnection
                                    if let Err(_) = socket.write_all(format!("{}\n", line).as_bytes()){
                                        //Write fail
                                        if let Err(_) = writeln!(std::io::stderr().lock(), "Write to socket failed") {
                                            //Print fail
                                        }
                                    }
                                }
                            }
                            return;
                        });
                        thread_handles.push(handle);
                    }
                }
            } else {
                break;
            }
        }
        for h in thread_handles {
             if let Err(_) = h.join() {
                 if let Err(_) = writeln!(std::io::stderr().lock(), "Thread panicked"){
                     // Print fail
                 }
             }
        }
    }
}
