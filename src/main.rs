use std::collections::HashMap;
use std::{env, fmt, io};
use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;
use std::string::ToString;

fn blank_ok() -> Result<String, String> {
    Ok("".to_string())
}

fn underflow_err() -> Result<String, String> {
    Err("Stack Underflow".to_string())
}

fn read_lines<P>(filename: P) -> io::Result<io::BufReader<File>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file))
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

struct State {
    defined_words: HashMap<String, String>,
    print_quote: bool,
}

fn main() -> Result<(), Error> {
    let mut stack = Vec::new();
    let mut state: State;
    state = State { defined_words: Default::default(), print_quote: false };

    // read in words from std (or file eventually) and evaluate
    let path = match std::env::args_os().nth(1) {
        Some(path) => std::path::PathBuf::from(path),
        None => {
            return Err(Error::from("Usage: cmd <path>"));
        }
    };
    let mut input: Box<dyn std::io::BufRead + 'static> = if path.as_os_str() == "-" {
        Box::new(std::io::stdin().lock())
    } else {
        match std::fs::File::open(&path) {
            Ok(file) => Box::new(BufReader::new(file)),
            Err(err) => {
                let msg = format!("{}: {}", path.display(), err);
                return Err(Error::from(msg));
            }
        }
    };

    for line in input.lines() {
        for word in line.unwrap_or("".to_string()).split(' ') {
            let result = run_word(&mut stack, &mut state, word);
            if result.is_ok() {
                let out = result.unwrap();
                if !out.is_empty() {
                    println!("{}", out)
                }
            } else {
                println!("{}", result.unwrap_err());
                return Err(Error::from("Unrecognized Word"));
            }
        }
    }
    return Ok(());
}

fn run_word(stack: &mut Vec<i64>, state: &mut State, word: &str) -> Result<String, String> {
    let int = word.parse::<i64>();
    if int.is_ok() {
        stack.push(int.unwrap());
        return Ok("".to_string());
    }

    if state.print_quote && word != "\"" {
        print!("{}", word);
        return blank_ok()
    }
    if state.print_quote && word == "\"" {
        println!();
        state.print_quote = false;
        return blank_ok()
    }

    // must be an actual word
    return match word.to_lowercase().as_str() {
        "+" => {
            if stack.len() < 2 {
                stack.clear()
            }
            let one = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            stack.push(one + two);
            blank_ok()
        }
        "." => {
            let result = stack.pop();
            if result.is_some() {
                println!("{}", result.unwrap());
                blank_ok()
            } else {
                underflow_err()
            }
        }
        "reset" => {
            //don't do a ton at this point, will be useful later
            stack.clear();
            blank_ok()
        }
        ".\"" => {
            state.print_quote = true;
            blank_ok()
        }
        _ => {
            if state.defined_words.contains_key(word) {
                //TODO defined stuff
            }

            Err("Unrecognized word ".to_string() + &word)
        }
    };
}
