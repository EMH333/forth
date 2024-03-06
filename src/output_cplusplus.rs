use crate::parsing::Word;

// this file is responsible for taking a vec of words, and outputting a c++ code string which runs the forth line
pub(crate) fn output_cplusplus(input: &Vec<Word>) -> String {
    let mut output: String = Default::default();

    //we know we always have some things we need to do to set up
    //namely, create the data and control stacks
    const HEADER: &str =
        "
#include <iostream>
#include <vector>
#define int64 int64_t

int64 pop(std::vector<int64> *stack) {
    int64 x = stack->back();
    stack->pop_back();
    return x;
}

int main() {
std::vector<int64> stack = {};

";
    output.push_str(HEADER);

    output.push_str("//from");
    output.push_str(&*format!("{:?}", *input));
    output.push_str("\n");

    // this will add the instructions for the state machine to run
    output.push_str(instruction_tape(&input).as_str());

    const FOOTER: &str = "std::cout << \" OK\\n\";}";
    output.push_str(FOOTER);

    output
}

// the actual switch statement to print the needed c++ code
fn instruction_tape(words: &&Vec<Word>) -> String {
    let mut output: String = "".to_string();
    for word in *words {
        match word {
            Word::Dup => {
                output.push_str("stack.push_back(stack.back());\n")
            }
            Word::Swap => {
                output.push_str("{int64 x = pop(&stack); int64 y = pop(&stack); stack.push_back(x); stack.push_back(y);}\n")
            }
            Word::Number(x) => {
                output.push_str(&*("stack.push_back(".to_owned() + x.to_string().as_str() + ");\n"))
            }
            Word::Dot => {
                output.push_str("printf(\"%ld\", pop(&stack));\n")
            }
            Word::Loop => {
                output.push_str("}\n")
            }
            Word::Do => {
                output.push_str("int64 index = pop(&stack); int64 limit = pop(&stack); for(int64 i = index; i < limit; i++){\n")
            }
            Word::If(_) => {
                output.push_str("if (pop(&stack) != 0) {\n");
            }
            Word::Else(_) => {
                output.push_str("} else {\n");
            }
            Word::Then => {
                output.push_str("}\n")
            }
            Word::Plus => {
                output.push_str("stack.push_back(pop(&stack) + pop(&stack));\n")
            }
            Word::Cr => {
                output.push_str("std::cout << '\\n';\n")
            }
            Word::Equal => {
                output.push_str("if (pop(&stack) == pop(&stack)) { stack.push_back(1); } else { stack.push_back(0); }\n")
            }
            Word::Greater => {
                todo!()
            }
            Word::Less => {
                todo!()
            }
            Word::MultDivide => {
                todo!()
            }
            Word::Mod => {
                output.push_str("{int64 two = pop(&stack); int64 one = pop(&stack); stack.push_back( one % two);}\n")
            }
            Word::Mult => {
                todo!()
            }
            Word::Drop => {
                output.push_str("stack.pop_back();\n")
            }
            Word::Rot => {
                output.push_str("{int64 three = pop(&stack); int64 two = pop(&stack); int64 one = pop(&stack); stack.push_back(two); stack.push_back(three); stack.push_back(one);}\n")
            }
            Word::I => {
                output.push_str("stack.push_back(i);\n")
            }
            Word::Quote(w) => {
                output.push_str(&*("std::cout << \"".to_owned() + w + "\";\n"))
            }
            Word::DoubleRot => {
                output.push_str("{int64 three = pop(&stack); int64 two = pop(&stack); int64 one = pop(&stack); stack.push_back(three); stack.push_back(one); stack.push_back(two);}\n")
            }
            Word::EqZero => {
                output.push_str("if (pop(&stack) == 0) { stack.push_back(1); } else { stack.push_back(0); }\n")
            }
            Word::NotIf(_) => {
                output.push_str("if (pop(&stack) == 0) {\n");
            }
            Word::DupModConst(n) => {
                output.push_str(&*("{int64 one = stack.back(); stack.push_back( one % ".to_owned() + n.to_string().as_str() + ");}\n"));
            }

            Word::Variable(_) => {
                todo!()
            }
            Word::Constant(_) => {
                todo!()
            }
            Word::J => {
                todo!()
            }

            //unsupported for now
            Word::Function(_) => {}
            Word::EndFunction => {}
            Word::Word(_) => {}
            Word::Reset => {}
            Word::Exclamation => {}
            Word::At => {}
            Word::UDotR => {}
        }
    }

    output
}
