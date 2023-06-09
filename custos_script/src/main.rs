use std::rc::Rc;

use custos_script::{
    bytecode,
    compiler::Compiler,
    parser::Parser,
    prelude::{BuiltInMethod, Constant, FunctionType, Instruction},
    tokenizer::Tokenizer,
    vm::VirtualMachine,
};

fn main() {
    let content = String::from(
        "
        func get_args {
            ret [1, 2, 3];
        }

        func main {
            var username = get_args()[0];
            if username == none
            {
                ret 1;
            } else {
                send(\"Your name is: \" + username);
            }
        }
        ",
    );

    let tokenizer = Tokenizer::new(&content);
    let mut parser = match Parser::new(tokenizer, &content) {
        Ok(p) => p,
        Err(e) => {
            panic!("{e}");
        }
    };
    match parser.parse() {
        Ok(_) => (),
        Err(e) => {
            panic!("{e}");
        }
    };

    let compiler = Compiler::default();
    let mut chunk = compiler.compile_non_boxed(parser.declarations);

    chunk.add_instruction(Instruction::GetGlobal("main".to_string()), 1);
    chunk.add_instruction(Instruction::Call(0), 1);
    chunk.add_instruction(Instruction::Return, 1);

    let mut vm = VirtualMachine::new(bytecode::Function {
        arity: 0,
        chunk,
        name: "".to_owned(),
        kind: FunctionType::Script,
    });

    vm.define_built_in_fn(BuiltInMethod::new(
        "send".to_owned(),
        Rc::new(move |_| Constant::None),
        0,
    ));

    if let Some(err) = vm.interpret() {
        panic!("{}", err)
    }
}
