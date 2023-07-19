use std::collections::HashMap;
use std::io::{BufRead, BufReader, LineWriter, stdout, Write};
use std::string::ToString;
use std::i64;

fn blank_ok() -> Result<InterpretResult, String> {
    Ok(InterpretResult::new_str(""))
}

fn underflow_err() -> Result<InterpretResult, String> {
    Err("Stack Underflow".to_string())
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, PartialEq)]
enum IfResult {
    DontCare,
    True,
    False,
}

#[derive(Debug)]
struct ControlStackFrame {
    index: i64,
    limit: i64,
    loop_start: i64,
    // the index of the line where the loop starts (after DO)
    if_result: IfResult,
}

#[derive(Debug)]
struct State {
    defined_words: HashMap<String, String>,
    variables: HashMap<String, i64>,
    // also serves constants
    control_stack: Vec<ControlStackFrame>,
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
    let mut stack = Vec::with_capacity(10);
    let mut state: State;
    state = State {
        defined_words: HashMap::with_capacity(5),
        variables: Default::default(),
        control_stack: Vec::with_capacity(3),
    };

    // read in words from std (or file eventually) and evaluate
    let path = match std::env::args_os().nth(1) {
        Some(path) => std::path::PathBuf::from(path),
        None => {
            return Err(Error::from("Usage: cmd <path>"));
        }
    };
    let input: Box<dyn BufRead + 'static> = if path.as_os_str() == "-" {
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

    let stdo = &mut stdout();
    let mut writer = Box::new(LineWriter::new((stdo) as &mut dyn Write)) as Box<LineWriter<&mut dyn Write>>;
    //let writer = out_writer.as_mut();//&mut LineWriter::new((stdout() as LineWriter<dyn Write>));
    for line in input.lines() {
        let l = line.unwrap();
        if l.is_empty() { continue; }
        let line_result = run_line(&mut stack, &mut state, l, writer.as_mut());
        if line_result.is_ok() {
            writer.flush().expect("Couldn't flush writer");
            print!(" OK")
        } else {
            return Err(line_result.unwrap_err());
        }
        println!()
    }
    return Ok(());
}

fn run_line(stack: &mut Vec<i64>, state: &mut State, line: String, writer: &mut LineWriter<&mut dyn Write>) -> Result<String, Error> {
    let words: Vec<&str> = line.split(' ').collect();

    let mut i = 0;

    while i < words.len() {
        let word = words.get(i).unwrap().to_lowercase().to_string();

        if word.is_empty() { continue; }

        if word == ".\"" {
            // now find the end, and print the whole thing
            let quote_last_index = skip_quote(i, &words);

            // grab the words between i and quote index, then concat and add to output
            let out = &words[i + 1..quote_last_index - 1].join(" ");
            writer.write_all(out.as_ref()).expect("Could not write output");

            i = quote_last_index;
            continue;
        }

        // functions
        if word == ":" {
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
            state.defined_words.insert(function_name.to_lowercase(), function.trim().to_string());
            i = func_index + 1;
            continue;
        }

        // variables
        if word == "variable" {
            //TODO err if last index isn't name of var
            //TODO figure out how variables are set/created, etc. I think it still just refers to the stack
            if stack.len() == 0 {
                stack.push(0);
            }
            let name = words.get(i + 1).unwrap();
            let loc = (stack.len() - 1) as i64; // use the stack location for now //TODO fix this later
            state.variables.insert(name.to_lowercase(), loc);

            i = i + 2;
            continue;
        }

        // constants
        if word == "constant" {
            //TODO err if last index isn't name of const
            if stack.len() == 0 {
                return Err(Error::from(underflow_err().unwrap_err()));
            }
            let name = words.get(i + 1).unwrap();
            let val = stack.pop().unwrap();
            state.variables.insert(name.to_lowercase(), val);

            i = i + 2;
            continue;
        }

        // loop
        if word == "loop" {
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

        // if
        if word == "if" {
            // see if true, otherwise skip it
            if stack.pop().unwrap() != 0 {
                //if true, then we pop it on the stack and continue
                state.control_stack.push(ControlStackFrame {
                    index: 0,
                    limit: 0,
                    loop_start: 0,
                    if_result: IfResult::True,
                });
                i += 1;
                continue;
            } else {
                //if false, then we pop it on the stack and look for another if, an else or then
                state.control_stack.push(ControlStackFrame {
                    index: 0,
                    limit: 0,
                    loop_start: 0,
                    if_result: IfResult::False,
                });


                let mut if_index = i + 1;
                let mut nested = 0;
                let mut if_word = words.get(if_index).unwrap().to_lowercase().to_string();
                while if_index < words.len() && (!(if_word == "else" || if_word == "then") || nested > 0) {
                    let new_index = skip_quote(if_index, &words);
                    if if_index == new_index {
                        // then we need to deal with a nested if scenario
                        if if_word == "if" {
                            nested += 1;
                        }
                        if if_word == "then" {
                            nested -= 1;
                        }
                        if_index += 1;
                    } else {
                        if_index = new_index;
                    }
                    if_word = words.get(if_index).unwrap().to_lowercase().to_string()
                }

                if if_index >= words.len() {
                    return Err(Error::from("No closing else or then"));
                }

                match if_word.as_str() {
                    "else" => {
                        // we want to run this, but keep the if on the control stack
                    }
                    "then" => {
                        // no else or other craziness, just pop the if from the stack and continue
                        state.control_stack.pop();
                    }
                    _ => todo!(),
                }

                // TODO if last index isn't else or then, error
                i = if_index + 1;
                continue;
            }
        }

        //else
        if word == "else" {
            // if if was false, continue as normal, otherwise skip it
            if state.control_stack.last().unwrap().if_result == IfResult::False {
                i += 1;
                continue;
            }

            let mut if_index = i + 1;
            let mut nested = 0;
            let mut else_word = words.get(if_index).unwrap().to_lowercase().to_string();
            while if_index < words.len() && (!(else_word == "then") || nested > 0) {
                let new_index = skip_quote(if_index, &words);
                if if_index == new_index {
                    // then we need to deal with a nested if scenario
                    if else_word == "if" {
                        nested += 1;
                    }
                    if else_word == "then" {
                        nested -= 1;
                    }
                    if_index += 1;
                } else {
                    if_index = new_index;
                }
                else_word = words.get(if_index).unwrap().to_lowercase().to_string()
            }

            if if_index >= words.len() {
                return Err(Error::from("No closing then"));
            }

            match else_word.as_str() {
                "then" => {
                    // no else or other craziness, just pop the if from the stack and continue
                    state.control_stack.pop();
                }
                _ => todo!(),
            }

            // TODO if last index isn't else or then, error
            i = if_index + 1;
            continue;
        }

        let result = run_word(stack, state, i as i64, &word, writer);
        if result.is_ok() {
            let out = result.unwrap();
            //output.push(out.output);
            writer.write_all(out.output.as_ref()).expect("Could not write out");
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
    return Ok("OK".to_string());
}

fn run_word(stack: &mut Vec<i64>, state: &mut State, index: i64, word: &String, output: &mut LineWriter<&mut dyn Write>) -> Result<InterpretResult, String> {
    let int = word.parse::<i64>();
    if int.is_ok() {
        stack.push(int.unwrap());
        return blank_ok();
    }

    //try to parse hex
    if word.starts_with("$") {
        let z = i64::from_str_radix(&word[1..], 16);
        return if z.is_ok() {
            stack.push(z.unwrap());
            blank_ok()
        } else {
            Err("Could not parse hex".to_string())
        };
    }

    //try to parse binary
    if word.starts_with("%") {
        let b = i64::from_str_radix(&word[1..], 2);
        return if b.is_ok() {
            stack.push(b.unwrap());
            blank_ok()
        } else {
            Err("Could not parse binary".to_string())
        };
    }

    // must be an actual word
    return match word.as_str() {
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
        "swap" => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(two);
            stack.push(one);
            blank_ok()
        }
        "rot" => {
            if stack.len() < 3 {
                return underflow_err();
            }

            let three = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(two);
            stack.push(three);
            stack.push(one);
            blank_ok()
        }
        "!" => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            if stack.len() < two as usize {
                return Err(format!("! address {} not in memory", two));
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
                if_result: IfResult::DontCare,
            };

            state.control_stack.push(frame);

            blank_ok()
        }
        "then" => {
            // must have come from an executed part of an if statement, safe to remove from control stack
            state.control_stack.pop();
            blank_ok()
        }
        "reset" => {
            //don't do a ton at this point, will be useful later
            stack.clear();
            state.variables.clear();
            state.control_stack.clear();
            blank_ok()
        }
        "\\" => {
            let mut out = InterpretResult::new_str("");
            out.skip_line = true;
            Ok(out)
        }
        _ => {
            if state.defined_words.contains_key(&*word) {
                let result = run_line(stack, state, String::from(state.defined_words.get(&*word).unwrap()), output);
                return if result.is_ok() {
                    blank_ok()
                } else {
                    Err(result.unwrap_err().to_string())
                };
            }

            if state.variables.contains_key(&*word) {
                stack.push(*state.variables.get(&*word).unwrap());
                return blank_ok();
            }

            Err("Unrecognized word ".to_string() + &word)
        }
    };
}

// given the current index, if the word is the start of a printed thing, returns the next clear index
fn skip_quote(current_index: usize, words: &Vec<&str>) -> usize {
    if words[current_index] == ".\"" {
        // now find the end, and print the whole thing
        let mut quote_index = current_index + 1;
        while quote_index < words.len() && words.get(quote_index).unwrap().to_string() != "\"" {
            quote_index += 1;
        }

        return quote_index + 1;
    }
    current_index
}