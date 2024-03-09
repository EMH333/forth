use std::str::FromStr;
use std::collections::HashMap;
use ahash::{HashSet, HashSetExt, RandomState};
use crate::{DefinedWord, State};

#[derive(PartialEq, Debug, Hash, Eq, Clone)]
pub(crate) enum Word {
    Dot,
    // name of function being declared
    Function(String),
    EndFunction,
    // name of var/const
    Variable(String),
    Constant(String),
    Loop,
    Do,
    // relative offset to else or then
    If(usize),
    // relative offset to end of statement
    Else(usize),
    Then,
    Plus,
    Cr,
    UDotR,
    Equal,
    Greater,
    Less,
    MultDivide,
    Mod,
    Mult,
    At,
    Dup,
    Drop,
    Swap,
    Rot,
    Exclamation,
    I,
    J,
    Reset,

    // word is a general placeholder, may be a variable, constant, function call, etc.
    Word(String),
    // numbers. Won't be parsed by from_string
    Number(i64),
    // quote. really means ." and "
    Quote(String),

    //special optimizations
    DoubleRot,
    EqZero,
    NotIf(usize),
    DupModConst(i64),
    DotCr,
}

impl FromStr for Word {
    type Err = ();

    fn from_str(input: &str) -> Result<Word, Self::Err> {
        match input.to_lowercase().as_str() {
            // needs to be filled in later
            ":" => Ok(Word::Function(String::new())),
            ";" => Ok(Word::EndFunction),
            "variable" => Ok(Word::Variable(String::new())),
            "constant" => Ok(Word::Constant(String::new())),
            "if" => Ok(Word::If(0)),
            "else" => Ok(Word::Else(0)),

            "loop" => Ok(Word::Loop),
            "do" => Ok(Word::Do),
            "." => Ok(Word::Dot),
            "then" => Ok(Word::Then),
            "+" => Ok(Word::Plus),
            "cr" => Ok(Word::Cr),
            "u.r" => Ok(Word::UDotR),
            "=" => Ok(Word::Equal),
            ">" => Ok(Word::Greater),
            "<" => Ok(Word::Less),
            "*/" => Ok(Word::MultDivide),
            "mod" => Ok(Word::Mod),
            "*" => Ok(Word::Mult),
            "@" => Ok(Word::At),
            "dup" => Ok(Word::Dup),
            "drop" => Ok(Word::Drop),
            "swap" => Ok(Word::Swap),
            "rot" => Ok(Word::Rot),
            "!" => Ok(Word::Exclamation),
            "i" => Ok(Word::I),
            "j" => Ok(Word::J),
            "reset" => Ok(Word::Reset),
            _ => Ok(Word::Word(input.to_string().clone())),
        }
    }
}

impl Word {
    fn new_quote(str: String) -> Word {
        Word::Quote(str)
    }

    fn new_num(num: i64) -> Word {
        Word::Number(num)
    }
}

pub(crate) fn parse_line(line: String) -> Result<Vec<Word>, String> {
    //parse numbers, quotes and the individual words
    // then go back and add the additional data (and compress function declarations, etc)
    let words: Vec<&str> = line.split(' ').collect();
    let mut out_words: Vec<Word> = Vec::with_capacity(words.len());

    let mut i = 0;

    while i < words.len() {
        let word = *words.get(i).unwrap();

        if word == "\\" || word == "" {
            break;
        }

        if word == ".\"" {
            // now find the end, and print the whole thing
            let quote_last_index = skip_quote(i, &words);

            // grab the words between i and quote index, then concat and add to output
            let out = &words[i + 1..quote_last_index - 1].join(" ");

            out_words.push(Word::new_quote((*out).clone()));

            i = quote_last_index;
            continue;
        }

        let int = word.parse::<i64>();
        if let Ok(val) = int {
            out_words.push(Word::new_num(val));
            i += 1;
            continue;
        }

        //try to parse hex
        if let Some(stripped) = word.strip_prefix('$') {
            let z = i64::from_str_radix(stripped, 16);
            if let Ok(val) = z {
                out_words.push(Word::new_num(val));
                i += 1;
                continue;
            } else {
                return Err("Could not parse hex".to_string());
            }
        }

        //try to parse binary
        if let Some(stripped) = word.strip_prefix('%') {
            let b = i64::from_str_radix(stripped, 2);
            if let Ok(val) = b {
                out_words.push(Word::new_num(val));
                i += 1;
                continue;
            } else {
                return Err("Could not parse binary".to_string());
            }
        }

        let parsed = Word::from_str(word);
        if let Ok(parsed_word) = parsed {
            out_words.push(parsed_word);

            i += 1;
            continue;
        } else {
            return Err("Could not parse word".to_string());
        }
    }

    // second pass figures out if, else, and function declarations
    let mut i = 0;
    while i < out_words.len() {
        let word = out_words.get(i).unwrap();
        match word {
            Word::If(_) => {
                //todo!("Look for else or then and store relative offset")
                let mut if_index = i + 1;
                let mut nested = 0;
                let mut if_word = out_words.get(if_index).unwrap();
                while if_index < out_words.len() && (!(matches!(if_word, Word::Else(_)) || matches!(if_word, Word::Then)) || nested > 0) {
                    // then we need to deal with a nested if scenario
                    if matches!(if_word, Word::If(_)) {
                        nested += 1;
                    }
                    if matches!(if_word, Word::Then) {
                        nested -= 1;
                    }

                    if_index += 1;
                    if_word = out_words.get(if_index).unwrap()
                }

                if if_index >= out_words.len() {
                    return Err("No closing else or then".to_string());
                }

                out_words[i] = Word::If(if_index - 1 - i);
            }
            Word::Else(_) => {
                //todo!("Look for then and store relative offset")
                let mut else_index = i + 1;
                let mut nested = 0;
                let mut else_word = out_words.get(else_index).unwrap();
                while else_index < out_words.len() && (!matches!(else_word,Word::Then) || nested > 0) {
                    // then we need to deal with a nested if scenario
                    if matches!(else_word, Word::If(_)) {
                        nested += 1;
                    }
                    if matches!(else_word, Word::Then) {
                        nested -= 1;
                    }

                    else_index += 1;
                    else_word = out_words.get(else_index).unwrap()
                }

                if else_index >= out_words.len() {
                    return Err("No closing else or then".to_string());
                }

                out_words[i] = Word::Else(else_index - 1 - i);
            }
            Word::Function(_) => {
                let next = out_words.get(i + 1).unwrap();
                match next {
                    Word::Word(x) => {
                        out_words[i] = Word::Function((*x).clone());
                        out_words.remove(i + 1);
                    }
                    _ => { return Err("Expected word after :".to_string()); }
                }
            }
            Word::Variable(_) => {
                let next = out_words.get(i + 1).unwrap();
                match next {
                    Word::Word(x) => {
                        out_words[i] = Word::Variable((*x).clone());
                        out_words.remove(i + 1);
                    }
                    _ => { return Err("Expected word after variable".to_string()); }
                }
            }
            Word::Constant(_) => {
                let next = out_words.get(i + 1).unwrap();
                match next {
                    Word::Word(x) => {
                        out_words[i] = Word::Constant((*x).clone());
                        out_words.remove(i + 1);
                    }
                    _ => { return Err("Expected word after constant".to_string()); }
                }
            }
            _ => {
                // do nothing on default case
            }
        }

        i += 1;
    }

    // third pass does optimizations
    optimization_pass(&mut out_words);


    Ok(out_words)
}

pub(crate) fn optimization_pass(out_words: &mut Vec<Word>) {
    let mut i = 0;
    while i < out_words.len() {
        let word = out_words.get(i).unwrap();
        match word {
            Word::Rot => {
                if let Some(next) = out_words.get(i + 1) {
                    if next == &Word::Rot {
                        out_words[i] = Word::DoubleRot;
                        out_words.remove(i + 1);
                    }
                }
            }
            Word::Number(0) => {
                let mut next = out_words.get(i + 1).unwrap();
                if next == &Word::Equal {
                    next = out_words.get(i + 2).unwrap();
                    if let Word::If(val) = next {
                        // if it is `0 = if`, then do notEquals optimization
                        out_words[i] = Word::NotIf(*val);
                        out_words.remove(i + 1);
                        out_words.remove(i + 1);//remove the extraneous operations
                    } else {
                        // if it isn't `0 = if`, then just do the eq zero optimization
                        out_words[i] = Word::EqZero;
                        out_words.remove(i + 1);
                    }
                }
            }
            Word::Dup => {
                // handle dup, number, mod (and eventually other operations)
                let mut next = out_words.get(i + 1).unwrap();
                if let Word::Number(val) = next{
                    next = out_words.get(i + 2).unwrap();
                    if next == &Word::Mod {
                        // if it is `dup x mod`, then do a constant mod optimization
                        out_words[i] = Word::DupModConst(*val);
                        out_words.remove(i + 1);
                        out_words.remove(i + 1);//remove the extraneous operations
                    }
                }
            }
            Word::Dot => {
                if let Some(next) = out_words.get(i + 1) {
                    if next == &Word::Cr {
                        out_words[i] = Word::DotCr;
                        out_words.remove(i + 1);
                    }
                }
            }
            _ => {
                // do nothing on default case
            }
        }

        i += 1;
    }
}

pub(crate) fn inline_function(func_name: &String, words: &Vec<Word>, defined_words: HashMap<String, DefinedWord, RandomState>) -> (Vec<Word>, HashSet<String>) {
    let mut output: Vec<Word> = Vec::with_capacity(words.len());
    let mut depends: HashSet<String> = HashSet::new();

    for word in words {
        match word {
            // inline functions if already defined
            Word::Word(raw_word) => {
                let defined_word = defined_words.get(raw_word);
                // don't inline recursion for now
                if raw_word == func_name {
                    output.push(word.clone())
                } else if let Some(cmd) = defined_word {
                    let command = cmd.clone();
                    output.append(&mut (*command.words).clone());
                    depends.insert(raw_word.clone());
                } else {
                    output.push(word.clone())
                }
            }
            _ => {
                output.push(word.clone());
            }
        }
    }

    //make sure and do an optimization pass
    optimization_pass(&mut output);

    return (output, depends);
}

// given the current index, if the word is the start of a printed thing, returns the next clear index
pub fn skip_quote(current_index: usize, words: &Vec<&str>) -> usize {
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

pub fn normalize_line(str: String) -> String {
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

pub(crate) fn break_inlining(func_name: String, state: &mut State) {
    for (name, word) in state.defined_words.clone() {
        // if it depends on the inlined word, then break the dependency (assuming not recursive call)
        if word.depends_on.contains(&*func_name) && name != func_name {
            let mut new = word.clone();
            new.words = new.original_words.clone();
            state.defined_words.insert(name.clone(), new);

            //also break inlining for the functions that depend on the checked function recursively
            break_inlining(name.clone(), state);
        }
    }
}