use custos_script::{
    prelude::{Chunk, Constant, Instruction, VariableManager},
    vm::VirtualMachine,
};

fn main() {
    let mut chunk = Chunk::default();
    let mut manager = VariableManager::default();

    manager.add_variable(&mut chunk, "test_var");
    manager.named_variable("test_var", false, &mut chunk);

    chunk.add_instruction(Instruction::Constant(Constant::Number(1.3)), 1);
    chunk.add_instruction(Instruction::Constant(Constant::Number(2.7)), 1);
    chunk.add_instruction(Instruction::Add, 1);

    chunk.add_instruction(Instruction::DefineGlobal(String::from("test_var")), 1);
    chunk.add_instruction(Instruction::GetGlobal(String::from("test_var")), 2);
    chunk.add_instruction(Instruction::Return, 3);

    let mut vm = VirtualMachine::new(chunk);
    vm.interpret();
}
