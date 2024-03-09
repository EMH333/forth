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
    const int64 x = stack->back();
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
                output.push_str("{const int64 size = stack.size(); const int64 two = stack[size-1]; const int64 one = stack[size-2]; stack[size-2] = two; stack[size-1] = one;}\n")
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
                output.push_str("{const int64 size = stack.size(); stack[size-2] += pop(&stack);}\n")
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
                // 1 2 3 => 2 3 1
                output.push_str("{const int64 size = stack.size(); const int64 three = stack[size-1]; const int64 two = stack[size-2]; const int64 one = stack[size-3]; stack[size-3] = two; stack[size-2] = three; stack[size-1] = one;}\n")
            }
            Word::I => {
                output.push_str("stack.push_back(i);\n")
            }
            Word::Quote(w) => {
                output.push_str(&*("std::cout << \"".to_owned() + w + "\";\n"))
            }
            Word::OnePlus => {
                output.push_str("{const int64 size = stack.size(); stack[size-1] += 1;}\n")
            }

            Word::DoubleRot => {
                // 1 2 3 => 3 1 2
                output.push_str("{const int64 size = stack.size(); const int64 three = stack[size-1]; const int64 two = stack[size-2]; const int64 one = stack[size-3]; stack[size-3] = three; stack[size-2] = one; stack[size-1] = two;}\n")
            }
            Word::EqZero => {
                output.push_str("if (stack.back() == 0) { stack[stack.size()-1] = 1; } else { stack[stack.size()-1] = 0; }\n")
            }
            Word::NotIf(_) => {
                output.push_str("if (pop(&stack) == 0) {\n");
            }
            Word::DupModConst(n) => {
                output.push_str(&*("stack.push_back( stack.back() % ".to_owned() + n.to_string().as_str() + ");\n"));
            }
            Word::DotCr => {
                output.push_str("printf(\"%ld\\n\", pop(&stack));\n")
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
            Word::PlusLoop => {
                //todo
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
