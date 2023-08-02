use std::str::FromStr;
use crate::skip_quote;

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
    If(i64),
    // relative offset to end of statement
    Else(i64),
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

        if word == "\\" {
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

                out_words[i] = Word::If((if_index - 1 - i) as i64);
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

                out_words[i] = Word::Else((else_index - 1 - i) as i64);
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

            Word::Rot => {
                let next = out_words.get(i + 1).unwrap();
                if next == &Word::Rot {
                    out_words[i] = Word::DoubleRot;
                    out_words.remove(i + 1);
                }
            }

            Word::Number(0) => {
                let next = out_words.get(i + 1).unwrap();
                if next == &Word::Equal {
                    out_words[i] = Word::EqZero;
                    out_words.remove(i + 1);
                }
            }
            _ => {
                // do nothing on default case
            }
        }

        i += 1;
    }

    Ok(out_words)
}