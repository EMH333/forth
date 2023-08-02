mod parsing;

use ahash::RandomState;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, stdout, Write};
use std::string::ToString;
use std::i64;
use std::rc::Rc;
use crate::parsing::{parse_line, Word};


fn underflow_err() -> Result<(), String> {
    Err("Stack Underflow".to_string())
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, PartialEq, Clone)]
enum IfResult {
    DontCare,
    True,
    False,
}

#[derive(Debug, Clone)]
struct ControlStackFrame {
    index: i64,
    limit: i64,
    loop_start: i64,
    // the index of the line where the loop starts (after DO)
    if_result: IfResult,
}

#[derive(Debug, Clone)]
struct DefinedWord {
    words: Rc<Vec<Word>>,
    has_been_inlined: bool,
    inline_count: i32, // number of times tried to inline, prevents recursion
}

#[derive(Debug, Clone)]
struct State {
    defined_words: HashMap<String, DefinedWord, RandomState>,
    variables: HashMap<String, i64, RandomState>,
    // also serves constants
    control_stack: Vec<ControlStackFrame>,
}

fn main() -> Result<(), Error> {
    let mut stack = Vec::with_capacity(10);
    let mut state: State;
    state = State {
        defined_words: HashMap::with_capacity_and_hasher(5, RandomState::new()),
        variables: HashMap::with_capacity_and_hasher(5, RandomState::new()),
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
    let mut writer = Box::new(BufWriter::new((stdo) as &mut dyn Write)) as Box<BufWriter<&mut dyn Write>>;
    //let writer = out_writer.as_mut();//&mut LineWriter::new((stdout() as LineWriter<dyn Write>));
    for line in input.lines() {
        let l = line.unwrap();
        if l.is_empty() { continue; }
        let parsed_line = parse_line(normalize_line(l).clone()).unwrap();
        //println!("{:?}", parsed_line);
        let line_result = run_line(&mut stack, &mut state, &parsed_line, writer.as_mut());
        if let Err(e) = line_result {
            return Err(e);
        } else {
            writer.flush().expect("Couldn't flush writer");
            print!(" OK")
        }
        println!()
    }
    Ok(())
}

fn normalize_line(str: String) -> String {
    // normalize string, by lower casing everything not going to be printed out
    let words: Vec<&str> = str.split(' ').collect();
    let mut output: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let quote_last_index = skip_quote(i, &words);
        if i == quote_last_index {
            output.push(words[i].to_lowercase());

            i += 1;
        } else {
            for word in words.iter().take(quote_last_index).skip(i) {
                output.push(word.to_string());
            }
            i = quote_last_index;
        }
    }
    output.join(" ")
}

fn inline_function(words: &Vec<Word>, defined_words: HashMap<String, DefinedWord, RandomState>) -> Vec<Word> {
    let mut output: Vec<Word> = Vec::with_capacity(words.len());

    for word in words {
        match word {
            // inline functions if already defined
            Word::Word(raw_word) => {
                let defined_word = defined_words.get(raw_word);
                if let Some(cmd) = defined_word {
                    let command = cmd.clone();
                    output.append(&mut (*command.words).clone());
                } else {
                    output.push(word.clone())
                }
            }
            _ => {
                output.push(word.clone());
            }
        }
    }

    output
}

fn run_line(stack: &mut Vec<i64>, state: &mut State, words: &Vec<Word>, writer: &mut BufWriter<&mut dyn Write>) -> Result<String, Error> {
    let mut i = 0;

    while i < words.len() {
        let word = words.get(i).unwrap();

        match word {
            Word::Quote(out) => {
                writer.write_all(out.as_ref()).expect("Could not write output");
            }
            Word::Function(function_name) => {
                // collect whole function, then resume later
                let mut func_index = i + 1;
                while func_index < words.len() && !matches!(words.get(func_index).unwrap(), Word::EndFunction) {
                    func_index += 1;
                }
                let function = &words[i + 1..func_index];
                // TODO if last index isn't ; then error
                state.defined_words.insert(function_name.clone(), DefinedWord {
                    words: Rc::new(function.to_vec()),
                    has_been_inlined: false,
                    inline_count: 0,
                });

                i = func_index
            }
            Word::Variable(name) => {
                //TODO err if last index isn't name of var
                //TODO figure out how variables are set/created, etc. I think it still just refers to the stack
                if stack.is_empty() {
                    stack.push(0);
                }
                let loc = (stack.len() - 1) as i64; // use the stack location for now //TODO fix this later
                state.variables.insert(name.clone(), loc);
            }
            Word::Constant(name) => {
                //TODO err if last index isn't name of const
                if stack.is_empty() {
                    return Err(Error::from(underflow_err().unwrap_err()));
                }
                let val = stack.pop().unwrap();
                state.variables.insert(name.clone(), val);
            }
            Word::Loop => {
                if state.control_stack.is_empty() {
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
            Word::If(next) => {
                // see if true, otherwise skip it
                if stack.pop().unwrap() != 0 {
                    //if true, then we pop it on the stack and continue
                    state.control_stack.push(ControlStackFrame {
                        index: 0,
                        limit: 0,
                        loop_start: 0,
                        if_result: IfResult::True,
                    });
                } else {
                    //if false, then we pop it on the stack and head to the offset
                    state.control_stack.push(ControlStackFrame {
                        index: 0,
                        limit: 0,
                        loop_start: 0,
                        if_result: IfResult::False,
                    });

                    // note, we are letting the i += 1 also run
                    i += *next as usize;
                }
            }
            Word::Else(next) => {
                // if it wasn't false, then skip, otherwise continue
                if state.control_stack.last().unwrap().if_result != IfResult::False {
                    // note, we are letting the i += 1 also run
                    i += *next as usize;
                }
            }
            // run everything else through run_word
            _ => {
                let result = run_word(stack, state, i as i64, word, writer);
                if let Err(e) = result {
                    println!("Err word: {:?}", word);
                    println!("{:?}", state);
                    println!("Stack: {:?}", stack);
                    return Err(Error::from(e));
                }
            }
        }

        i += 1;
    }
    Ok("OK".to_string())
}

fn run_word(stack: &mut Vec<i64>, state: &mut State, index: i64, word: &Word, output: &mut BufWriter<&mut dyn Write>) -> Result<(), String> {
    // must be an actual word
    match word {
        Word::Number(n) => {
            stack.push(*n);
        }
        Word::Plus => {
            if stack.len() < 2 {
                return underflow_err();
            }
            let one = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            stack.push(one + two);
        }
        Word::Dot => {
            let result = stack.pop();
            if let Some(val) = result {
                write!(output, "{}", val).expect("Could not write value");
            } else {
                return underflow_err();
            }
        }
        Word::Cr => {
            writeln!(output).expect("Could not write out newline");
        }
        Word::UDotR => {
            //TODO properly implement
        }
        Word::Equal => {
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
        }
        Word::Greater => {
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
        }
        Word::Less => {
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
        }
        Word::MultDivide => {
            if stack.len() < 3 {
                return underflow_err();
            }

            let three = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            let inter = one * two;
            stack.push(inter / three);
        }
        Word::Mod => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(one % two);
        }
        Word::Mult => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(one * two);
        }
        Word::At => {
            if stack.is_empty() {
                return underflow_err();
            }

            let one = stack.pop().unwrap();

            if stack.len() < one as usize {
                return underflow_err(); //TODO make this an actual different error
            }

            stack.push(*stack.get(one as usize).unwrap());
        }
        Word::Dup => {
            if stack.is_empty() {
                return underflow_err();
            }

            let one = stack.last().unwrap(); // don't pop if unneeded
            stack.push(*one);
        }
        Word::Drop => {
            if stack.is_empty() {
                return underflow_err();
            }

            stack.pop();
        }
        Word::Swap => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(two);
            stack.push(one);
        }
        Word::Rot => {
            if stack.len() < 3 {
                return underflow_err();
            }

            let three = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(two);
            stack.push(three);
            stack.push(one);
        }
        Word::Exclamation => {
            if stack.len() < 2 {
                return underflow_err();
            }

            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            if stack.len() < two as usize {
                return Err(format!("! address {} not in memory", two));
            }

            stack[two as usize] = one;
        }
        Word::I => {
            if state.control_stack.is_empty() {
                return underflow_err();
            }
            stack.push(state.control_stack.last().unwrap().index);
        }
        Word::J => {
            if state.control_stack.len() < 2 {
                return underflow_err();
            }
            stack.push(state.control_stack[state.control_stack.len() - 2].index);
        }
        Word::Do => {
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
        }
        Word::Then => {
            // must have come from an executed part of an if statement, safe to remove from control stack
            state.control_stack.pop();
        }
        Word::Reset => {
            //don't do a ton at this point, will be useful later
            stack.clear();
            state.variables.clear();
            state.control_stack.clear();
        }
        Word::Word(raw_word) => {
            let defined_word = state.defined_words.get(raw_word);
            if let Some(cmd) = defined_word {
                let command = cmd.clone();
                let result = run_line(stack, state, &command.words, output);

                // this is a slow path, but that's fine because it is only run a few times per function
                // note the 16 here prevents functions from being unrolled recursively
                if !command.has_been_inlined && command.inline_count < 16 {
                    let output = inline_function(&command.words, state.clone().defined_words);
                    let len = output.len();
                    state.defined_words.insert(raw_word.clone(), DefinedWord {
                        words: Rc::new(output),
                        has_been_inlined: len == command.words.len(), // only consider a function fully inlined if the size doesn't change
                        inline_count: command.inline_count + 1,
                    });
                }

                if let Err(e) = result {
                    return Err(e.to_string());
                }

                return Ok(());
            }

            if state.variables.contains_key(raw_word) {
                stack.push(*state.variables.get(raw_word).unwrap());
                return Ok(());
            }

            return Err("Unrecognized word ".to_string() + &*format!("{:?}", word));
        }

        //optimizations
        Word::DoubleRot => {
            if stack.len() < 3 {
                return underflow_err();
            }

            let three = stack.pop().unwrap();
            let two = stack.pop().unwrap();
            let one = stack.pop().unwrap();

            stack.push(three);
            stack.push(one);
            stack.push(two);
        }
        Word::EqZero => {
            if stack.is_empty() {
                return underflow_err();
            }

            let one = stack.pop().unwrap();

            if one == 0 {
                stack.push(1)
            } else {
                stack.push(0)
            }
        }
        _ => { return Err("Can't handle ".to_string() + &*format!("{:?}", word)); }
    }

    Ok(())
}

// given the current index, if the word is the start of a printed thing, returns the next clear index
fn skip_quote(current_index: usize, words: &Vec<&str>) -> usize {
    if words[current_index] == ".\"" {
        // now find the end, and print the whole thing
        let mut quote_index = current_index + 1;
        while quote_index < words.len() && *words.get(quote_index).unwrap() != "\"" {
            quote_index += 1;
        }

        return quote_index + 1;
    }
    current_index
}