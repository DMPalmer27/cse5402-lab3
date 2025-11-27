/*
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: play.rs
 * Summary: This file contains the Play struct and its implementation. A Play is the
 * type used for coordinating the script generation of the play. It handles the 
 * individual scenes as instances of the Scene Fragment structs and is responsible
 * for populating them such that they can each fulfill their individual role of 
 * managing specific characters in each scene.
 * 
 */

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use super::scene_fragment::SceneFragment;
use super::declarations;


type ScriptConfig = Vec<(bool, String)>;

const SCENE_INDICATOR: &str = "[scene]";
const EMPTY: usize = 0;
const SINGLE_TOKEN: usize = 1;
const FIRST_TOKEN: usize = 0;
const SECOND_TOKEN: usize = 1;
const NEW_SCENE_BOOL: bool = true;
const CONFIG_FILE_BOOL: bool = false;
const FIRST_FRAGMENT: usize = 0;
const SECOND_FRAGMENT: usize = 1;
const START: usize = 0;


macro_rules! poison_mutex_print {
    () => {
        match writeln!(std::io::stderr().lock(), "Error: mutex was poisoned and could not be accessed") {
            Ok(_) => {} //success
            Err(_) => {} //fail
        }
    };
}


pub struct Play {
    fragments: Vec<Arc<Mutex<SceneFragment>>>,
}


impl Play {
    pub fn new() -> Self {
        Self {
            fragments: Vec::new(),
        }
    }

    // This function processes a passed in ScriptConfig. For each item in the ScriptConfig if it contains a scene title it updates the title and otherwise creates a new SceneFragment, adds it to the Play's fragments, and prepares the fragment with its associated file. If it fails, the error is propagated out and otherwise Ok(()) is returned
    fn process_config(&mut self, script_config: &ScriptConfig) -> Result<(), u8> {
        let mut title  = String::new();
        let mut thread_handles = Vec::new();
        for tup in script_config {
            match tup {
                (true, text) => { //Text is a new title
                    title = text.clone();
                },
                (false, text) => {
                    let text = text.to_string();
                    let mut frag = SceneFragment::new(&title);
                    let handle = thread::spawn( move || -> SceneFragment{
                        frag.prepare(&text);
                        frag
                    });
                    title = "".to_string();

                    thread_handles.push(handle);
                }
            }
        }
        for h in thread_handles {
            match h.join() {
                Err(_) => {
                    return Err(declarations::ERR_SCRIPT_GEN)
                } //thread panicked
                Ok(frag) => {
                    self.fragments.push(Arc::new(Mutex::new(frag)));
                }
            }
        }
        Ok(())
    }

    // This function separates the tokens in the passed in line, creating a new scene if the first
    // token is [scene] and there is a scene title after. Otherwise, treats the first token as a
    // config file. In either success case an element containing the info is pushed to the passed
    // in ScriptConfig, and in the event of an empty line or [scene] is the first token with
    // nothing after nothing is pushed.
    fn add_config(line: &str, script_config: &mut ScriptConfig) {
        let trimmed = line.trim();
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() == EMPTY {
            return;
        }
        if tokens.len() == SINGLE_TOKEN && tokens[FIRST_TOKEN] == SCENE_INDICATOR {
            use std::sync::atomic::Ordering;
            if declarations::WHINGE_ON.load(Ordering::SeqCst){
                match writeln!(std::io::stderr().lock(), "Warning: scene identified but has no title so has not been added") {
                    Ok(_) => {}, //success
                    Err(_) => {}, //fail
                }
            }
            return;
        }
        if tokens[FIRST_TOKEN] == SCENE_INDICATOR {
            let rest = tokens[SECOND_TOKEN..].join(" ");
            script_config.push((NEW_SCENE_BOOL, rest));
        } else {
            script_config.push((CONFIG_FILE_BOOL, tokens[FIRST_TOKEN].to_string()));
            if tokens.len() != SINGLE_TOKEN{
                use std::sync::atomic::Ordering;
                if declarations::WHINGE_ON.load(Ordering::SeqCst) {
                    match writeln!(std::io::stderr().lock(), "Warning: there are additional tokens in the line \"{}\" that is being treated as a config file name", line){
                        Ok(_) => {}, //success
                        Err(_) => {}, //fail
                    }
                }
            }
        }
            
    }



    // This function reads a given script file name and populates the passed in 
    // script_config with the relevant information from this config file. It propagates any errors
    // out and otherwise returns Ok(())
    fn read_config(script_file_name: &str, script_config: &mut ScriptConfig) -> Result<(), u8> {
        let mut lines: Vec<String> = Vec::new();
        declarations::grab_trimmed_file_lines(script_file_name, &mut lines)?;
        if lines.len() == EMPTY {
            match writeln!(std::io::stderr().lock(), "Error: the script gen file must contain at least 1 line"){
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
            return Err(declarations::ERR_SCRIPT_GEN);
        }
        for line in &lines {
            Self::add_config(line, script_config);
        }
        Ok(())
    }


    // This method does the script generation for a given play. It uses the above functions to
    // populate the self Play with associated information.
    pub fn prepare(&mut self, script_file_name: &str) -> Result<(), u8> {
        let mut script_config: ScriptConfig = Default::default();
        Self::read_config(script_file_name, &mut script_config)?;
        self.process_config(&script_config)?;
        if self.fragments.len() != EMPTY {
            match self.fragments[FIRST_FRAGMENT].lock() {
                Ok(ref frag_guard) => {
                    if !frag_guard.scene_title.is_empty() { 
                        Ok(()) 
                    } else {
                        match writeln!(std::io::stderr().lock(), "Error: script generation failed") {
                            Ok(_) => {}, //success
                            Err(_) => {}, //fail
                        }
                        Err(declarations::ERR_SCRIPT_GEN)
                    }
                }
                Err(_) => {
                    poison_mutex_print!();
                    Err(declarations::ERR_MUTEX)
                }
            }
        } else {
            match writeln!(std::io::stderr().lock(), "Error: script generation failed"){
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
            Err(declarations::ERR_SCRIPT_GEN)
        }
    }


    // This function prints the script by iterating over each scene fragment and printing
    // everything required for it, including character entrances, exits, and lines.
    pub fn recite(&mut self) { 
        let len = self.fragments.len();
        for i in START..len {
            // Generate disjoint slices of self.fragments so that you can get a mutable reference
            // to the frag at index i and immutable references to the before and after frags
            let (before, rest) = self.fragments.split_at_mut(i);
            let (frag, after) = rest.split_at_mut(SECOND_FRAGMENT);

            let prev_arc = if i > START {Some(&before[i-1])} else {None};
            let next_arc = if i < len - 1 {Some(&after[FIRST_FRAGMENT])} else {None};

            match frag[FIRST_FRAGMENT].lock() {
                Ok(ref mut frag_guard) => {
                    if let Some(p) = prev_arc {
                        match p.lock() {
                            Ok(ref p_guard) => {
                                frag_guard.enter(p_guard);
                            }
                            Err(_) => {
                                poison_mutex_print!();
                            }
                        }
                    } else {
                        frag_guard.enter_all();
                    }

                    frag_guard.recite();

                    if let Some(n) = next_arc {
                        match n.lock() {
                            Ok(ref n_guard) => {
                                frag_guard.exit(n_guard);
                            }
                            Err(_) => {
                                poison_mutex_print!();
                            }
                        }
                    } else {
                        frag_guard.exit_all();
                    }
                }
                Err(_) => {
                    poison_mutex_print!();
                }
            }

        }

    }

}
