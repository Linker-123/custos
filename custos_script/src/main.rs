use custos_script::{
    prelude::{Chunk, Constant, Instruction},
    vm::VirtualMachine,
};

fn main() {
    let mut chunk = Chunk::default();
    chunk.add_instruction(Instruction::Constant(Constant::Number(2.)), 1);
    chunk.add_instruction(Instruction::Constant(Constant::Number(0.)), 1);
    chunk.add_instruction(Instruction::Divide, 1);
    chunk.add_instruction(Instruction::Return, 2);

    let mut vm = VirtualMachine::new(chunk);
    vm.interpret();
}
