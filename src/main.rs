mod parsing;
mod output_cplusplus;

use ahash::{HashSet, HashSetExt, RandomState};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, stdout, Write};
use std::string::ToString;
use std::i64;
use std::rc::Rc;
use crate::output_cplusplus::output_cplusplus;
use crate::parsing::{parse_line, Word};

const MAX_CONTROL_LENGTH: usize = 1000;

fn underflow_err() -> Result<(), String> {
    Err("Stack Underflow".to_string())
}
fn control_stack_overflow_err() -> Result<(), String> {
    Err("Control Stack Overflow".to_string())
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

impl ControlStackFrame {
    fn new_if(res: IfResult) -> ControlStackFrame {
        ControlStackFrame {
            index: 0,
            limit: 0,
            loop_start: 0,
            if_result: res,
        }
    }
}

#[derive(Debug, Clone)]
struct DefinedWord {
    words: Rc<Vec<Word>>,
    original_words: Rc<Vec<Word>>, // so we can revert inlining
    has_been_inlined: bool,
    inline_count: i32, // number of times tried to inline, prevents recursion
    depends_on: HashSet<String>, // the defined words that have been inlined into this word
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
        let parsed_line = parse_line(parsing::normalize_line(l).clone()).unwrap();

        try_output_cpp(&parsed_line, &state);

        //println!("{:?}", parsed_line);
        let line_result = run_line(&mut stack, &mut state, &parsed_line, writer.as_mut());
        if let Err(e) = line_result {
            return Err(e);
        } else {
            writer.flush().expect("Couldn't flush writer");
            print!(" OK")
        }
        println!();
    }
    Ok(())
}

fn try_output_cpp(parsed_line: &Vec<Word>, state: &State) {
    if parsed_line.len() == 1 {
        let func = parsed_line[0].clone();
        match func {
            Word::Word(x) => {
                //make sure it's fully optimized and loop till it is
                if let Some(line) = state.defined_words.get(&*x) {
                    let to_use: Vec<Word>;
                    if !line.has_been_inlined {
                        let mut previous_len = 999;
                        let mut out: Vec<Word> = line.words.to_vec();
                        while out.len() != previous_len {
                            previous_len = out.len();
                            let (o, _) = parsing::inline_function(&x, &out, state.defined_words.clone());
                            out = o;
                            //println!("Optimized {:?} to\n {:?}", line.words, out)
                        }
                        to_use = out;
                    } else {
                        to_use = line.words.to_vec()
                    }
                    let output = output_cplusplus(&to_use);
                    println!("{}", output);
                } else {
                    println!("Word needs to be defined to generate cpp")
                }
            }
            _ => {}//panic!("Only supports single word line calling a function when printing c++")
        }
    }
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
                    original_words: Rc::new(function.to_vec()),
                    has_been_inlined: false,
                    inline_count: 0,
                    depends_on: HashSet::new(),
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
                if let Some(mut last) = state.control_stack.pop() {
                    last.index += 1;

                    if last.index < last.limit {
                        i = last.loop_start as usize;
                        state.control_stack.push(last);
                    } else {
                        i += 1;
                    }
                    continue;
                } else {
                    return Err(Error::from(underflow_err().unwrap_err()));
                }
            }
            Word::If(next) => {
                if state.control_stack.len() > MAX_CONTROL_LENGTH {
                    return Err(Error::from(control_stack_overflow_err().unwrap_err()));
                }
                
                // see if true, otherwise skip it
                if stack.pop().unwrap() != 0 {
                    //if true, then we pop it on the stack and continue
                    state.control_stack.push(ControlStackFrame::new_if(IfResult::True));
                } else {
                    //if false, then we pop it on the stack and head to the offset
                    state.control_stack.push(ControlStackFrame::new_if(IfResult::False));

                    // note, we are letting the i += 1 also run
                    i += *next;
                }
            }
            // this is an optimization
            Word::NotIf(next) => {
                if state.control_stack.len() > MAX_CONTROL_LENGTH {
                    return Err(Error::from(control_stack_overflow_err().unwrap_err()));
                }

                // see if false (as in, the stack is equal to zero), otherwise skip it
                if stack.pop().unwrap() == 0 {
                    //if true, then we pop it on the stack and continue
                    state.control_stack.push(ControlStackFrame::new_if(IfResult::True));
                } else {
                    //if false, then we pop it on the stack and head to the offset
                    state.control_stack.push(ControlStackFrame::new_if(IfResult::False));

                    // note, we are letting the i += 1 also run
                    i += *next;
                }
            }
            Word::Else(next) => {
                // if it wasn't false, then skip, otherwise continue
                if state.control_stack.last().unwrap().if_result != IfResult::False {
                    // note, we are letting the i += 1 also run
                    i += *next;
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
            if let Some(one) = stack.last() {
                stack.push(*one);
            } else {
                return underflow_err();
            }
        }
        Word::Drop => {
            if stack.pop().is_none() {
                return underflow_err();
            }
        }
        Word::Swap => {
            let len = stack.len();
            if len < 2 {
                return underflow_err();
            }
            stack.swap(len - 2, len - 1);
        }
        Word::Rot => {
            let len = stack.len();
            if len < 3 {
                return underflow_err();
            }

            stack.swap(len - 1, len - 2);//one, three, two
            stack.swap(len - 3, len - 1);//two, three, one
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
            if let Some(last) = state.control_stack.last() {
                stack.push(last.index)
            } else {
                return underflow_err();
            }
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
                    let (output, mut depends) = parsing::inline_function(raw_word, &command.words, state.defined_words.clone());
                    let len = output.len();

                    command.depends_on.iter().for_each(|f| _ = depends.insert(f.clone()));

                    state.defined_words.insert(raw_word.clone(), DefinedWord {
                        words: Rc::new(output),
                        original_words: command.original_words.clone(),
                        has_been_inlined: len == command.words.len(), // only consider a function fully inlined if the size doesn't change
                        inline_count: command.inline_count + 1,
                        depends_on: depends,
                    });

                    //now we undo all the inlining that depends on this word to preserve correctness
                    parsing::break_inlining(raw_word.clone(), state);
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
            let len = stack.len();
            if len < 3 {
                return underflow_err();
            }

            stack.swap(len - 1, len - 2);//one, three, two
            stack.swap(len - 3, len - 2);//three, one, one
        }
        Word::EqZero => {
            if let Some(one) = stack.pop() {
                if one == 0 {
                    stack.push(1)
                } else {
                    stack.push(0)
                }
            } else {
                return underflow_err();
            }
        }
        _ => { return Err("Can't handle ".to_string() + &*format!("{:?}", word)); }
    }

    Ok(())
}