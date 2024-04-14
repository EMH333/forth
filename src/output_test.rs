#[cfg(test)]
mod tests {
    use std::io::Write;
    use ahash::{HashMap, RandomState};
    use crate::parsing::parse_line;
    use crate::{Error, parsing, run_line, State};


    fn get_output_from_line(l: String) -> Result<String, Error> {
        let mut stack = Vec::with_capacity(10);
        let mut state: State;
        state = State {
            defined_words: HashMap::with_capacity_and_hasher(10, RandomState::new()),
            variables: HashMap::with_capacity_and_hasher(10, RandomState::new()),
            if_control_stack: Vec::with_capacity(10),
            loop_control_stack: Vec::with_capacity(10),
            internal_buffer: Vec::with_capacity(10),
        };

        let mut buffer: Vec<u8> = Vec::new();
        let parsed_line = parse_line(parsing::normalize_line(l).clone()).unwrap();

        let line_result = run_line(&mut stack, &mut state, &parsed_line, &mut buffer as &mut dyn Write);
        return if let Err(e) = line_result {
            Err(e)
        } else {
            Ok(String::from_utf8(buffer)?)
        };
    }

    #[test]
    fn basic_line() {
        let result = get_output_from_line(".\" hello world \"".to_string());
        assert_eq!(result.unwrap(), "hello world".to_string())
    }

    #[test]
    fn nested_ifs_1() {
        let input = "1 1 IF .\" First \" IF .\" Second \" THEN ELSE .\" Else \" IF .\" True In Else \" THEN THEN";
        let result = get_output_from_line(input.to_string());
        assert_eq!(result.unwrap(), "FirstSecond".to_string())
    }

    #[test]
    fn nested_ifs_2() {
        let input = "1 0 IF .\" First \" IF .\" Second \" THEN ELSE .\" Else \" IF .\" True In Else \" THEN THEN";
        let result = get_output_from_line(input.to_string());
        assert_eq!(result.unwrap(), "ElseTrue In Else".to_string())
    }

    #[test]
    fn nested_ifs_3() {
        let input = "0 1 IF .\" First \" IF .\" Second \" THEN ELSE .\" Else \" IF .\" True In Else \" THEN THEN";
        let result = get_output_from_line(input.to_string());
        assert_eq!(result.unwrap(), "First".to_string())
    }

    #[test]
    fn nested_ifs_4() {
        let input = "0 0 IF .\" First \" IF .\" Second \" THEN ELSE .\" Else \" IF .\" True In Else \" THEN THEN";
        let result = get_output_from_line(input.to_string());
        assert_eq!(result.unwrap(), "Else".to_string())
    }

    #[test]
    fn fizzbuzz() {
        let input = "16 1 DO I DUP 3 MOD 0 = IF .\" fizz \" 1 ELSE 0 THEN SWAP 5 MOD 0 = IF .\" buzz \" 1+ THEN 0 = IF I . THEN CR LOOP";
        let output = "1\n2\nfizz\n4\nbuzz\nfizz\n7\n8\nfizz\nbuzz\n11\nfizz\n13\n14\nfizzbuzz\n";
        let result = get_output_from_line(input.to_string());
        assert_eq!(result.unwrap(), output.to_string());
    }

    #[test]
    fn fizzbuzz_alternative() {
        let input = "15 1 DO I . CR I 1+ . CR .\" fizz \" CR I 3 + . CR .\" buzz \" CR .\" fizz \" CR I 6 + . CR I 7 + . CR .\" fizz \" CR .\" buzz \" CR I 10 + . CR .\" fizz \" CR I 12 + . CR I 13 + . CR .\" fizzbuzz \" CR 15 +LOOP";
        let output = "1\n2\nfizz\n4\nbuzz\nfizz\n7\n8\nfizz\nbuzz\n11\nfizz\n13\n14\nfizzbuzz\n";
        let result = get_output_from_line(input.to_string());
        assert_eq!(result.unwrap(), output.to_string());
    }
}