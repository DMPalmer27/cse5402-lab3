/*
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: scene_fragment.rs
 * Summary: This file holds the Scene Frament struct and its implementation. The 
 * Scene Fragment is used to coordinate the printing of a scene, announcing all 
 * characters in the scene and ensuring that they all give their lines properly. 
 *
 */

use std::collections::HashSet;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::cmp::Ordering;

use super::player::Player;
use super::declarations;


type PlayConfig = Vec<(String, String)>; // (character name, associated text file)

const CHARACTER_NAME: usize = 0;
const CHARACTER_FILE: usize = 1;
const CONFIG_LINE_TOKENS: usize = 2;
const MIN_CONFIG_LINES: usize = 2;
const FIRST_LINE: usize = 0;
const EMPTY: usize = 0;
const EXPECTED_NUM_SPEAKERS: usize = 1;

macro_rules! poison_mutex_print {
    () => {
        match writeln!(std::io::stderr().lock(), "Error: mutex was poisoned and could not be accessed") {
            Ok(_) => {} //success
            Err(_) => {} //fail
        }
    };
}


pub struct SceneFragment {
    pub scene_title: String,
    characters: Vec<Arc<Mutex<Player>>>,
}


impl SceneFragment {
    pub fn new(title: &str) -> Self {
        Self {
            scene_title: title.to_string(),
            characters: Vec::new(),
        }
    }

    // This function processes a passed in PlayConfig. For each item in the PlayConfig it creates a
    // Player, adds it to the Play's characters, and prepares the character with its associated
    // text file. 
    // If it fails the error is propagated out and otherwise Ok(()) is returned
    fn process_config(&mut self, play_config: &PlayConfig) -> Result<(), u8> {
        for tup in play_config {
            match tup {
                (name, file) => {
                    let mut character = Player::new(&name);
                    character.prepare(&file)?;
                    self.characters.push(Arc::new(Mutex::new(character)));
                }
            }
        }
        Ok(())
    }

    // This function splits the passed in line into two separate tokens and adds them as a tuple to
    // the passed in PlayConfig. If the tokens could not be properly extracted and whinge mode is
    // on it complains, but if there were at least two tokens (the minimum amount) it adds the
    // line.
    fn add_config(line: &str, play_config: &mut PlayConfig) {
        let delimited_tokens: Vec<&str> = line.split_whitespace().collect();
        if delimited_tokens.len() != CONFIG_LINE_TOKENS {
            use std::sync::atomic::Ordering;
            if declarations::WHINGE_ON.load(Ordering::SeqCst) {
                match writeln!(std::io::stderr().lock(), "Warning: there were not exactly two distinct tokens in the line {}", line) {
                    Ok(_) => {}, //success
                    Err(_) => {}, //fail
                }
            }
        }
        if delimited_tokens.len() >= CONFIG_LINE_TOKENS {
            play_config.push((
                    delimited_tokens[CHARACTER_NAME].to_string(),
                    delimited_tokens[CHARACTER_FILE].to_string()
                    ));
        }
    }



    // This function reads a given config file name and populates the passed in title and
    // play_config with the relevant information from this config file. It propagates any errors
    // out and otherwise returns Ok(())
    fn read_config(config_file_name: &str, play_config: &mut PlayConfig) -> Result<(), u8> {
        let mut lines: Vec<String> = Vec::new();
        declarations::grab_trimmed_file_lines(config_file_name, &mut lines)?;
        if lines.len() < MIN_CONFIG_LINES {
            match writeln!(std::io::stderr().lock(), "Error: the config file must contain at least one character and associated text file") {
                Ok(_) => {}, //success
                Err(_) => {},//fail
            }
            return Err(declarations::ERR_SCRIPT_GEN);
        }
        for line in &lines {
            Self::add_config(line, play_config);
        }
        Ok(())
    }


    // This method does the script generation for a given scene. It uses the above functions to
    // populate the self Play with associated information.
    pub fn prepare(&mut self, config_file_name: &str) -> Result<(), u8> {
        let mut play_config: PlayConfig = Default::default();
        Self::read_config(config_file_name, &mut play_config)?;
        self.process_config(&play_config)?;
        self.characters.sort_by(SceneFragment::compare_players);
        Ok(())
    }


    // This method prints the play line by line by finding the player that has the next line and
    // printing it out.
    pub fn recite(&mut self) {
        let mut next_line_number = FIRST_LINE;
        let mut cur_speaker = String::new();
        loop {
            let min_line_number = match self.characters
                .iter()
                .filter_map(|c| {
                    match c.lock() {
                        Ok(ref c_guard) => c_guard.next_line(),
                        Err(_) => {
                            poison_mutex_print!();
                            None
                        }
                    }
                })
                .min(){
                Some(n) => n,
                None => break,
            };
            
            // Skip over any missing line numbers, complaining if whinge mode is on
            while min_line_number > next_line_number {
                use std::sync::atomic::Ordering;
                if declarations::WHINGE_ON.load(Ordering::SeqCst) {
                    match writeln!(std::io::stderr().lock(), "Warning: missing line {}", next_line_number) {
                        Ok(_) => {}, //success
                        Err(_) => {}, //fail
                    }
                }
                next_line_number += 1;
            }

            let mut num_speakers = EMPTY;
            for c in &self.characters {
                match c.lock() {
                    Ok(ref mut c_guard) => {
                        if c_guard.next_line() == Some(min_line_number) {
                            c_guard.speak(&mut cur_speaker);
                            num_speakers += 1;
                        }
                    }
                    Err(_) => {
                        poison_mutex_print!();
                    }
                }
            }
            
            if num_speakers != EXPECTED_NUM_SPEAKERS {
                use std::sync::atomic::Ordering;
                if declarations::WHINGE_ON.load(Ordering::SeqCst) {
                    match writeln!(std::io::stderr().lock(), "Warning: there are {} characters who have a line with number {}", num_speakers, min_line_number) {
                        Ok(_) => {}, //success
                        Err(_) => {}, //fail
                    }
                }
            }
            next_line_number += 1;
        }
    }

    // This function announces all characters in self but not in other for scene transitions
    pub fn enter(&self, other: &Self) {
        if !self.scene_title.trim().is_empty(){
            match writeln!(std::io::stdout().lock(), "\n{}\n", self.scene_title){
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
        }
        let other_names: HashSet<String> = other.characters.iter()
            .filter_map(|c| {
                match c.lock() {
                    Ok(ref c_guard) => Some(c_guard.name.clone()),
                    Err(_) => {
                        poison_mutex_print!();
                        None
                    }
                }
            })
            .collect();
        for name in self.characters.iter().filter_map(|c| {
            match c.lock() {
                Ok(ref c_guard) => Some(c_guard.name.clone()),
                Err(_) => {
                    poison_mutex_print!();
                    None
                }
            }
        }) {
            if !other_names.contains(&name) {
                match writeln!(std::io::stdout().lock(), "[Enter {}.]", name) {
                    Ok(_) => {}, //success
                    Err(_) => {}, //fail
                }
            }
        }
        
    }
    // This function announces the entrance of all characters in self
    pub fn enter_all(&self) {
        if !self.scene_title.trim().is_empty(){
            match writeln!(std::io::stdout().lock(), "\n{}\n", self.scene_title){
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
        }
        for name in self.characters.iter().filter_map(|c| {
            match c.lock() {
                Ok(ref c_guard) => Some(c_guard.name.clone()),
                Err(_) => {
                    poison_mutex_print!();
                    None
                }
            }
        }) {
            match writeln!(std::io::stdout().lock(), "[Enter {}.]", name) {
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
        }
    }

    // This function announces the exit of characters in self but not in other. This is so
    // that only the characters who are actually exiting are announced as such.
    pub fn exit(&self, other: &Self) {
        let other_names: HashSet<String> = other.characters.iter()
            .filter_map(|c| {
                match c.lock() {
                    Ok(ref c_guard) => Some(c_guard.name.clone()),
                    Err(_) => {
                        poison_mutex_print!();
                        None
                    }
                }
            })
            .collect();
        match writeln!(std::io::stdout().lock()) {
            Ok(_) => {}, //success
            Err(_) => {}, //fail
        }
        for name in self.characters.iter().rev().filter_map(|c| {
            match c.lock() {
                Ok(ref c_guard) => Some(c_guard.name.clone()),
                Err(_) => {
                    poison_mutex_print!();
                    None
                }
            }
        }) {
            if !other_names.contains(&name) {
                match writeln!(std::io::stdout().lock(), "[Exit {}.]", name){
                    Ok(_) => {}, //success
                    Err(_) => {}, //fail
                }
            }
        }
        match writeln!(std::io::stdout().lock()) {
            Ok(_) => {}, //success
            Err(_) => {}, //fail
        }
    }

    // This function announces the exit of all characters in self
    pub fn exit_all(&self) {
        match writeln!(std::io::stdout().lock()) {
            Ok(_) => {}, //success
            Err(_) => {}, //fail
        }
        for name in self.characters.iter().rev().filter_map(|c| {
            match c.lock() {
                Ok(ref c_guard) => Some(c_guard.name.clone()),
                Err(_) => {
                    poison_mutex_print!();
                    None
                }
            }
        }) {
            match writeln!(std::io::stdout().lock(), "[Exit {}.]", name) {
                Ok(_) => {}, //success
                Err(_) => {}, //fail
            }
        }
        match writeln!(std::io::stdout().lock()) {
            Ok(_) => {}, //success
            Err(_) => {}, //fail
        }
    }

    // This function returns the ordering between two players in a thread safe way by acquiring the
    // required mutexes and then calling the partial comparison implentation for the underlying
    // Player
    pub fn compare_players(a: &Arc<Mutex<Player>>, b: &Arc<Mutex<Player>>) -> Ordering {
        let a_lock = a.lock();
        let b_lock = b.lock();
        match (a_lock, b_lock) {
            (Ok(ref a_guard), Ok(ref b_guard)) => {
                match Player::partial_cmp(a_guard, b_guard) {
                    Some(order) => order,
                    _ => Ordering::Equal,
                }
            }
            _ => Ordering::Equal,
        }
    }

}
