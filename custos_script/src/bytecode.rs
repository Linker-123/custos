#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    Bool(bool),
    String(String),
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
#[non_exhaustive]
pub enum Instruction {
    Constant(Constant),
    Add,
    Subtract,
    Multiply,
    Divide,
    DefineGlobal(String),
    SetGlobal(String),
    GetGlobal(String),
    GetLocal(usize),
    SetLocal(usize),
    Pop,
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

#[derive(Debug, Clone)]
pub struct LocalVariable {
    name: String,
    depth: usize,
}

#[derive(Debug, Clone, Default)]
pub struct VariableManager {
    locals: Vec<LocalVariable>,
    pub scope_depth: usize,
}

impl VariableManager {
    pub fn start_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self, chunk: &mut Chunk) {
        self.scope_depth -= 1;

        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            chunk.add_instruction(Instruction::Pop, 0);
            self.locals.pop();
        }
    }

    pub fn mark_intialized_last(&mut self) {
        if let Some(local) = self.locals.last_mut() {
            local.depth = self.scope_depth;
        }
    }

    /// you MUST add the bytecode value of the variable before calling this function
    pub fn add_variable(&mut self, chunk: &mut Chunk, name: &str) {
        if self.scope_depth > 0 {
            // we are in a scope.
            // we don't add anything to the chunk because the value itself is already on the stack,
            // locals do not have names at runtime, they are retrieved by their index.
            self.locals.push(LocalVariable {
                name: name.to_owned(),
                depth: self.scope_depth,
            });
        } else {
            // TODO: line tracking
            chunk.add_instruction(Instruction::DefineGlobal(name.to_owned()), 0);
            chunk.add_instruction(Instruction::Constant(Constant::String(name.to_owned())), 0);
        }
    }

    pub fn named_variable(&self, name: &str, is_set: bool, chunk: &mut Chunk) {
        let local_index = self.resolve_local(name);

        // Ugly but it's better than copying name 2 times using to_owned and defining
        // the instructions in separate variables
        if let Some(stack_idx) = local_index {
            if is_set {
                chunk.add_instruction(Instruction::SetLocal(stack_idx), 0);
            } else {
                chunk.add_instruction(Instruction::GetLocal(stack_idx), 0);
            }
        } else if is_set {
            chunk.add_instruction(Instruction::GetGlobal(name.to_owned()), 0);
        } else {
            chunk.add_instruction(Instruction::SetGlobal(name.to_owned()), 0);
        }
    }

    pub fn resolve_local(&self, name: &str) -> Option<usize> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(index);
            }
        }
        None
    }
}
