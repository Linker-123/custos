use custos_script::{
    prelude::{Chunk, Constant, Function, FunctionType, Instruction, VariableManager},
    vm::VirtualMachine,
};

fn main() {
    let mut script_chunk = Chunk::default();
    let mut manager = VariableManager::default();

    let mut inner_chunk = Chunk::default();

    // STATEMENT: 2.1 + 5.6;
    inner_chunk.add_instruction(Instruction::Constant(Constant::Number(2.1)), 1); // define 2.1
    inner_chunk.add_instruction(Instruction::Constant(Constant::Number(5.6)), 1); // define 5.6
    inner_chunk.add_instruction(Instruction::Add, 1); // sum them up and store the result on stack
    inner_chunk.add_instruction(Instruction::Pop, 1); // pop the result off stack because ;

    inner_chunk.add_instruction(Instruction::Constant(Constant::None), 1);
    inner_chunk.add_instruction(Instruction::Return, 1); // return from the function

    script_chunk.add_instruction(
        Instruction::Constant(Constant::Function(Function {
            chunk: inner_chunk,
            arity: 0,
            name: "test".to_owned(),
            kind: FunctionType::Function,
        })),
        10,
    );

    manager.add_variable(&mut script_chunk, "test");

    let mut inner_chunk = Chunk::default();
    // STATEMENT: 2.1 * 5.6;
    inner_chunk.add_instruction(Instruction::Constant(Constant::Number(2.1)), 1); // define 2.1
    inner_chunk.add_instruction(Instruction::Constant(Constant::Number(5.6)), 1); // define 5.6
    inner_chunk.add_instruction(Instruction::Multiply, 1); // sum them up and store the result on stack
    inner_chunk.add_instruction(Instruction::Pop, 1); // pop the result off stack because ;

    inner_chunk.add_instruction(Instruction::Constant(Constant::None), 1);
    inner_chunk.add_instruction(Instruction::Return, 1); // return from the function

    script_chunk.add_instruction(
        Instruction::Constant(Constant::Function(Function {
            chunk: inner_chunk,
            arity: 0,
            name: "test2".to_owned(),
            kind: FunctionType::Function,
        })),
        10,
    );

    manager.add_variable(&mut script_chunk, "test2");

    manager.named_variable("test", false, &mut script_chunk);

    script_chunk.add_instruction(Instruction::Call(0), 2);
    script_chunk.add_instruction(Instruction::Pop, 2);

    manager.named_variable("test2", false, &mut script_chunk);

    script_chunk.add_instruction(Instruction::Call(0), 2);
    script_chunk.add_instruction(Instruction::Pop, 2);

    script_chunk.add_instruction(Instruction::Constant(Constant::None), 1);
    script_chunk.add_instruction(Instruction::Return, 2);

    let mut vm = VirtualMachine::new(Function {
        arity: 0,
        chunk: script_chunk,
        name: "".to_owned(),
        kind: FunctionType::Script,
    });
    vm.interpret();
}
