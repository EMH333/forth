use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::string::ToString;

fn blank_ok() -> Result<InterpretResult, String> {
    Ok(InterpretResult::new_str(""))
}

fn underflow_err() -> Result<InterpretResult, String> {
    Err("Stack Underflow".to_string())
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

struct State {
    defined_words: HashMap<String, String>,
    print_quote: bool,
}

#[derive(Debug)]
struct InterpretResult {
    output: String,
    skip_line: bool,
}

impl InterpretResult {
    fn new(out: String) -> InterpretResult {
        InterpretResult {
            output: out,
            skip_line: false,
        }
    }

    fn new_str(out: &str) -> InterpretResult {
        InterpretResult {
            output: out.to_string(),
            skip_line: false,
        }
    }
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
    let input: Box<dyn std::io::BufRead + 'static> = if path.as_os_str() == "-" {
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
        let line_result = run_line(&mut stack, &mut state, line);
        if line_result.is_ok() {
            for out in line_result.unwrap() {
                print!("{}", out)
            }
        } else {
            return Err(line_result.unwrap_err())
        }
        println!()
    }
    return Ok(());
}

fn run_line(stack: &mut Vec<i64>, state: &mut State, line: std::io::Result<String>) -> Result<Vec<String>, Error> {
    let mut output = Vec::new();
    let unwrapped_line = line.unwrap_or("".to_string());
    let words:Vec<&str> = unwrapped_line.split(' ').collect();
    for i in 0..words.len() {
        let word = words.get(i).unwrap();
        let result = run_word(stack, state, word);
        if result.is_ok() {
            let out = result.unwrap();
            output.push(out.output);
            if out.skip_line {
                break
            }
        } else {
            println!("Err word: {}", word);
            return Err(Error::from(result.unwrap_err()));
        }
    }
    return Ok(output)
}

//TODO maybe have a word stack as well, instead of state. might need to have a state machine state var as well with like " or : as the contents
fn run_word(stack: &mut Vec<i64>, state: &mut State, word: &str) -> Result<InterpretResult, String> {
    if state.print_quote && word != "\"" {
        print!("{} ", word);
        return blank_ok()
    }
    if state.print_quote && word == "\"" {
        println!();
        state.print_quote = false;
        return blank_ok()
    }

    let int = word.parse::<i64>();
    if int.is_ok() {
        stack.push(int.unwrap());
        return blank_ok();
    }

    // must be an actual word
    return match word.to_lowercase().as_str() {
        "+" => {
            if stack.len() < 2 {
                stack.clear() //TODO should this be an under flow?
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
        "=" => {
            if stack.len() < 2 {
                return underflow_err()
            }

            let one = stack.pop().unwrap();
            let two = stack.pop().unwrap();

            if one == two {
                stack.push(1)
            } else {
                stack.push(0)
            }
            blank_ok()
        }
        ">" => {
            if stack.len() < 2 {
                return underflow_err()
            }

            let one = stack.pop().unwrap();
            let two = stack.pop().unwrap();

            if one < two {
                stack.push(1)
            } else {
                stack.push(0)
            }
            blank_ok()
        }
        "<" => {
            if stack.len() < 2 {
                return underflow_err()
            }

            let one = stack.pop().unwrap();
            let two = stack.pop().unwrap();

            if one > two {
                stack.push(1)
            } else {
                stack.push(0)
            }
            blank_ok()
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
        "\\" => {
            let mut out = InterpretResult::new_str("");
            out.skip_line = true;
            Ok(out)
        }
        _ => {
            if state.defined_words.contains_key(word) {
                //TODO defined stuff
            }

            Err("Unrecognized word ".to_string() + &word)
        }
    };
}
