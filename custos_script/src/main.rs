use custos_script::{parser::Parser, tokenizer::Tokenizer};

fn main() {
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

    //     let mut vm = VirtualMachine::new(Function {
    //         arity: 0,
    //         chunk: script_chunk,
    //         name: "".to_owned(),
    //         kind: FunctionType::Script,
    //     });
    //     vm.interpret();
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
        func my_func(x):
            var x = 3
        end

        my_func()
    "
    .to_owned();
    let tokenizer = Tokenizer::new(&binding);
    let mut parser = Parser::new(tokenizer, &binding);
    parser.parse();

    for decl in parser.declarations {
        println!("{decl:#?}");
    }
}
