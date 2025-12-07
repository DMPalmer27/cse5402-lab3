/*
 * Author: Daniel Palmer
 * Email: d.m.palmer@wustl.edu
 * File: player.rs
 * Summary: This file contains the Player struct and its implementation. A Player is
 * the type primarily used for each character in a scene and is responsible for 
 * parsing and preparing the character's lines from the associated file and for 
 * printing out the line.
 *
 */

use std::cmp::Ordering;
use std::io::Write;

use super::declarations;

const EMPTY: usize = 0;
const FIRST_LINE: usize = 0;


pub type PlayLines = Vec<(usize, String)>; // (line number, string)

pub struct Player {
    pub name: String,
    lines: PlayLines,
    line_index: usize,
}

//This implementation block creates all associated methods and functions for the player
impl Player {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            lines: PlayLines::new(),
            line_index: EMPTY,
        }
    }

    // This method parses a line to add to a Player's lines, separating the line number from the
    // content before adding tuple containing these items into the Player's lines. It raises
    // warnings if parsing fails and the line should not be added
    fn add_script_line(&mut self, unparsed_line: &str) {
        if unparsed_line.len() > 0{
            if let Some((first_token, rest)) = unparsed_line.split_once(char::is_whitespace) {
                let first_token_trim = first_token.trim();
                let rest_trim = rest.trim();

                match first_token_trim.parse::<usize>() {
                    Ok(num) => self.lines.push((num, rest_trim.to_string())),
                    Err(_) => {
                        use std::sync::atomic::Ordering;
                        if declarations::WHINGE_ON.load(Ordering::SeqCst) {
                            if let Err(_) = writeln!(std::io::stderr().lock(), "Warning: {} does not contain a valid usize value", first_token_trim) {
                                // Print fail
                            }
                        }
                    },
                }
            } else {
                use std::sync::atomic::Ordering;
                if declarations::WHINGE_ON.load(Ordering::SeqCst) {
                    if let Err(_) = writeln!(std::io::stderr().lock(), "Warning: line contains only a single token and is invalid") {
                        // Print fail
                    }
                }
            }
        }
    }

    // This method adds the lines from a character's part file into the character's Player struct
    // lines field
    pub fn prepare(&mut self, file_name: &str) {
        let mut lines: Vec<String> = Vec::new();
        if let Err(_) = declarations::grab_trimmed_file_lines(file_name, &mut lines){
            panic!("Failed to read file");
        }
        for line in &lines {
            self.add_script_line(line);
        }
        self.lines.sort();
    }

    // This method speaks the character's next line. If the character was not previously speaking,
    // it introduces the character by printing their name before printing the desired line
    pub fn speak(&mut self, recent_player: &mut String) {
        if self.line_index < self.lines.len() {
            if *recent_player != self.name {
                *recent_player = self.name.clone();
                if let Err(_) = writeln!(std::io::stdout().lock(), "\n {}", self.name){
                    // Print fail
                }
            }
            let (_, line) = &self.lines[self.line_index];
            if let Err(_) = writeln!(std::io::stdout().lock(), "{}", line) {
                // Print fail
            }
            self.line_index += 1;
        }
    }

    // This method returns an option containing the line_index of the next line to speak if it
    // exists and None otherwise 
    pub fn next_line(&self) -> Option<usize> {
        if self.line_index < self.lines.len() {
            let (line_num, _) = &self.lines[self.line_index];
            Some(*line_num)
        } else {
            None
        }
    }
}

// This block implements the partial equivalent trait for the player which allows the comparison of
// players to decide which to introduce first
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        let self_silent = self.lines.len() == 0;
        let other_silent = other.lines.len() == 0;
        if self_silent && other_silent {
            true
        } else if self_silent || other_silent {
            false
        } else {
            let (self_first, _) = self.lines[FIRST_LINE];
            let (other_first, _) = other.lines[FIRST_LINE];
            self_first == other_first
        }
    }
}

// This block implements the equivalent trait for the player which is done in the partial
// equivalent block above. So, there is no new logic here
impl Eq for Player{}

// This block implements the partial ordering trait for the player which allows more
// complicated comparison of players to decide which to introduce first. The ordering is complete, 
// so partial ordering should just wrap the result of cmp in Some
impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// This block implements the strict ordering trait for the player which, same as partial ordreing,
// alows more complicated comparison of players to decide which to introduce first. It checks if
// any of the players are silent because this is the "greatest" value and, if both characters are
// not silent, detects which has the first line.
impl Ord for Player {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_silent = self.lines.len() == 0;
        let other_silent = other.lines.len() == 0;

        match (self_silent, other_silent) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => {
                let (self_first, _) = self.lines[FIRST_LINE];
                let (other_first, _) = other.lines[FIRST_LINE];
                self_first.cmp(&other_first)
            },
        }
    }
}
