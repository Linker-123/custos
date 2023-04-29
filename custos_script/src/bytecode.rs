#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    Bool(bool),
    String(f64),
}

impl Constant {
    pub fn get_pretty_type(&self) -> &'static str {
        match self {
            Constant::Number(_) => "number",
            Constant::Bool(_) => "boolean",
            Constant::String(_) => "string",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Constant(Constant),
    Add,
    Subtract,
    Multiply,
    Divide,
    Return,
}

impl Instruction {
    pub fn print_ins(&self, line: usize) {
        println!("{:04}\t{:?}", line, self);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<Instruction>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn add_instruction(&mut self, instruction: Instruction, line: usize) {
        self.code.push(instruction);
        self.lines.push(line);
    }
}

impl std::ops::Index<usize> for Chunk {
    type Output = Instruction;

    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}
