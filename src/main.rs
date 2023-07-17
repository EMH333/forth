use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::string::ToString;
use std::i64;

fn blank_ok() -> Result<InterpretResult, String> {
    Ok(InterpretResult::new_str(""))
}

fn underflow_err() -> Result<InterpretResult, String> {
    Err("Stack Underflow".to_string())
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct ControlStackFrame {
    index: i64,
    limit: i64,
    loop_start: i64, // the index of the line where the loop starts (after DO)
}

#[derive(Debug)]
struct State {
    defined_words: HashMap<String, String>,
    variables: HashMap<String, i64>,
    // also serves constants
    control_stack: Vec<ControlStackFrame>,
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
    state = State { defined_words: Default::default(), variables: Default::default(), control_stack: vec![], print_quote: false };

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
        let line_result = run_line(&mut stack, &mut state, line.unwrap_or("".to_string()));
        if line_result.is_ok() {
            for out in line_result.unwrap() {
                print!("{}", out)
            }
        } else {
            return Err(line_result.unwrap_err());
        }
        println!()
    }
    return Ok(());
}

fn run_line(stack: &mut Vec<i64>, state: &mut State, line: String) -> Result<Vec<String>, Error> {
    let mut output = Vec::new();
    let words: Vec<&str> = line.split(' ').collect();

    let mut i = 0;

    while i < words.len() {
        let word = words.get(i).unwrap();

        if word.is_empty() { continue; }

        // functions
        if word.to_string() == ":" && !state.print_quote {
            // collect whole function, then resume later
            let mut function: String = String::new();
            let mut function_name: String = String::new();
            let mut func_index = i + 1;
            while func_index < words.len() && words.get(func_index).unwrap().to_string() != ";" {
                // get function name
                if i + 1 == func_index {
                    function_name = words.get(func_index).unwrap().to_string();
                } else {
                    function.push(' ');
                    function.push_str(words.get(func_index).unwrap());
                }

                func_index += 1;
            }
            // TODO if last index isn't ; then error
            state.defined_words.insert(function_name, function.trim().to_string());
            i = func_index + 1;
            continue;
        }

        // variables
        if word.to_lowercase().to_string() == "variable" && !state.print_quote {
            //TODO err if last index isn't name of var
            //TODO figure out how variables are set/created, etc. I think it still just refers to the stack
            if stack.len() == 0 {
                stack.push(0);
            }
            let name = words.get(i + 1).unwrap();
            let loc = (stack.len() - 1) as i64; // use the stack location for now //TODO fix this later
            state.variables.insert(name.to_string(), loc);

            i = i + 2;
            continue;
        }

        // constants
        if word.to_lowercase().to_string() == "constant" && !state.print_quote {
            //TODO err if last index isn't name of const
            if stack.len() == 0 {
                return Err(Error::from(underflow_err().unwrap_err()));
            }
            let name = words.get(i + 1).unwrap();
            let val = stack.pop().unwrap();
            state.variables.insert(name.to_string(), val);

            i = i + 2;
            continue;
        }

        // loop
        if word.to_lowercase().to_string() == "loop" && !state.print_quote {
            if state.control_stack.len() < 1 {
                return Err(Error::from(underflow_err().unwrap_err()));
            }

            let mut last = state.control_stack.pop().unwrap();
            last.index += 1;

            if last.index < last.limit {
                i = last.loop_start as usize;
                state.control_stack.push(last);
            } else {
                i += 1;
            }
            continue;
        }

        let result = run_word(stack, state, i as i64, word);
        if result.is_ok() {
            let out = result.unwrap();
            output.push(out.output);
            if out.skip_line {
                break;
            }
        } else {
            println!("Err word: {}", word);
            println!("{:?}", state);
            println!("Stack: {:?}", stack);
            return Err(Error::from(result.unwrap_err()));
        }

        i += 1;
    }
    return Ok(output);
}

//TODO maybe have a word stack as well, instead of state. might need to have a state machine state var as well with like " or : as the contents
fn run_word(stack: &mut Vec<i64>, state: &mut State, index: i64, word: &str) -> Result<InterpretResult, String> {
    if state.print_quote && word != "\"" {
        print!("{} ", word);
        return blank_ok();
    }
    if state.print_quote && word == "\"" {
        println!();
        state.print_quote = false;
        return blank_ok();
    }

    let int = word.parse::<i64>();
    if int.is_ok() {
        stack.push(int.unwrap());
        return blank_ok();
    }

    //try to parse hex
    let without_prefix = word.trim_start_matches("$");
    let z = i64::from_str_radix(without_prefix, 16);
    if z.is_ok() {
        stack.push(z.unwrap());
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
                return Ok(InterpretResult::new(result.unwrap().to_string()));
            } else {
                underflow_err()
            }
        }
        "cr" => {
            return Ok(InterpretResult::new_str("\n"));
        }
        "u.r" => {
            //TODO properly implement
            blank_ok()
        }
        "=" => {
            if stack.len() < 2 {
                return underflow_err();
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
                return underflow_err();
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
                return underflow_err();
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
        "*/" => {
            if stack.len() < 3 {
                return underflow_err();
            }

            let three = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            let inter = one * two;
            stack.push(inter / three);
            blank_ok()
        }
        "mod" => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(one % two);
            blank_ok()
        }
        "*" => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(one * two);
            blank_ok()
        }
        "@" => {
            if stack.len() < 1 {
                return underflow_err();
            }

            let one = stack.pop().unwrap();

            if stack.len() < one as usize {
                return underflow_err(); //TODO make this an actual different error
            }

            stack.push(*stack.get(one as usize).unwrap());
            blank_ok()
        }
        "dup" => {
            if stack.len() < 1 {
                return underflow_err();
            }

            let one = stack.pop().unwrap();

            stack.push(one);
            stack.push(one);

            blank_ok()
        }
        "drop" => {
            if stack.len() < 1 {
                return underflow_err();
            }

            stack.pop();

            blank_ok()
        }
        "!" => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            if stack.len() < two as usize {
                return underflow_err(); //TODO make this an actual different error
            }

            stack[two as usize] = one;
            blank_ok()
        }
        "i" => {
            if state.control_stack.len() < 1 {
                return underflow_err();
            }
            stack.push(state.control_stack.last().unwrap().index);
            blank_ok()
        }
        "j" => {
            if state.control_stack.len() < 2 {
                return underflow_err();
            }
            stack.push(state.control_stack[state.control_stack.len() - 2].index);
            blank_ok()
        }
        "do" => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            let frame = ControlStackFrame {
                index: two,
                limit: one,
                loop_start: index + 1,
            };

            state.control_stack.push(frame);

            blank_ok()
        }
        "reset" => {
            //don't do a ton at this point, will be useful later
            stack.clear();
            state.variables.clear();
            state.control_stack.clear();
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
                let result = run_line(stack, state, String::from(state.defined_words.get(word).unwrap()));
                return if result.is_ok() {
                    let mut out = String::new();
                    for s in result.unwrap() {
                        out += &*s;
                    }
                    Ok(InterpretResult::new(out))
                } else {
                    Err(result.unwrap_err().to_string())
                };
            }

            if state.variables.contains_key(word) {
                stack.push(*state.variables.get(word).unwrap());
                return blank_ok();
            }

            Err("Unrecognized word ".to_string() + &word)
        }
    };
}
