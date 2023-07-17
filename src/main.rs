use std::collections::HashMap;
use std::io;
use std::io::BufRead;

fn main() {
    println!("Hello, world!");

    let mut stack = Vec::new();
    let mut defined_words : HashMap<String, String> = HashMap::new();

    // read in words from std (or file eventually) and evaluate
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        for word in line.unwrap_or("".to_string()).split(' ') {
            let result = run_word(&mut stack, &mut defined_words, word);
            if result.is_ok() {
                println!("{}", result.unwrap())
            } else {
                println!("{}", result.unwrap_err())
            }
        }
    }
}

fn run_word(stack: &mut Vec<i64>, defined_words : &mut HashMap<String, String>, word: &str) -> Result<String, String> {
    let int = word.parse::<i64>();
    if int.is_ok() {
        stack.push(int.unwrap());
        return Ok("".to_string());
    }

    // must be an actual word
    return match word {
        "+" => {
            if stack.len() < 2 {
                stack.clear()
            }
            let one = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            stack.push(one + two);
            Ok("".to_string())
        }
        "." => {
            let result = stack.pop();
            if result.is_some() {
                println!("{}", result.unwrap());
                return Ok("".to_string())
            } else {
                Err("Stack Underflow".to_string())
            }
        }
        _ => {
            Err("Unrecognized word".to_string() + &word)
        }
    }
}
