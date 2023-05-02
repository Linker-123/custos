use custos_script::{
    compiler::Compiler,
    parser::Parser,
    prelude::{BuiltInMethod, Constant, Function, FunctionType, Instruction},
    tokenizer::Tokenizer,
    vm::VirtualMachine,
};

fn main() {
    env_logger::init();
    // THE FOLLOWING IS EQUIVALENT TO:
    // if (2 == 2) { let then_block = 20 } else { let else_block = 10 }
    // const TIMES: u16 = 20000;
    // let mut times: Vec<Duration> = Vec::with_capacity(TIMES as usize);

    // for _ in 0..TIMES {
    //     let mut script_chunk = Chunk::default();
    //     // let mut manager = VariableManager::default();

    //     script_chunk.add_instruction(Instruction::Constant(Constant::Number(2.)), 1);
    //     script_chunk.add_instruction(Instruction::Constant(Constant::Number(2.)), 1);
    //     script_chunk.add_instruction(Instruction::Equal, 1);

    //     script_chunk.add_instruction(Instruction::JumpIfFalse(1), 1);
    //     script_chunk.add_instruction(Instruction::Pop, 1); // Pop the comparing expression

    //     // this is incorrect in a real case scenario, but do i care?
    //     script_chunk.add_instruction(Instruction::Constant(Constant::Number(20.)), 1);
    //     script_chunk.add_instruction(Instruction::DefineGlobal("then_block".to_owned()), 1);

    //     // else jump
    //     script_chunk.add_instruction(Instruction::Jump(3), 1);

    //     let conditional_jump_ins = &mut script_chunk[3];
    //     *conditional_jump_ins = Instruction::JumpIfFalse(4); // 4 instructions after JumpIfFalse

    //     script_chunk.add_instruction(Instruction::Pop, 1);

    //     script_chunk.add_instruction(Instruction::Constant(Constant::Number(10.)), 1);
    //     script_chunk.add_instruction(Instruction::DefineGlobal("else_block".to_owned()), 1);

    //     script_chunk.add_instruction(Instruction::Constant(Constant::None), 1);
    //     script_chunk.add_instruction(Instruction::Return, 2);

    //     let start = Instant::now();

    //     times.push(start.elapsed());
    // }

    // let mut sum: u128 = 0;
    // for time in &times {
    //     sum += time.as_micros();
    // }

    // println!(
    //     "Average {:.2?}",
    //     Duration::from_micros((sum / (times.len() as u128)) as u64)
    // )

    let binding = "
        func sum(a, b):
            var x = 3
            var result = a + b + x
            ret result
        end

        func main:
            // var x = sum(1, 2)
            print(1, \"amogus\")
        end
    "
    .to_owned();
    let tokenizer = Tokenizer::new(&binding);
    let mut parser = Parser::new(tokenizer, &binding);
    parser.parse();

    let compiler = Compiler::default();
    let mut chunk = compiler.compile_non_boxed(parser.declarations);

    chunk.add_instruction(Instruction::GetGlobal("main".to_string()), 1);
    chunk.add_instruction(Instruction::Call(0), 1);
    chunk.add_instruction(Instruction::Return, 1);

    // chunk.print_chunk();

    let mut vm = VirtualMachine::new(Function {
        arity: 0,
        chunk,
        name: "".to_owned(),
        kind: FunctionType::Script,
    });

    vm.define_built_in_fn(BuiltInMethod::new(
        "print".to_owned(),
        |args| {
            for arg in args {
                print!("{} ", arg);
            }
            println!();
            Constant::None
        },
        0u8,
    ));
    vm.interpret();
}
