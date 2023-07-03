use std::{collections::VecDeque, rc::Rc};

#[derive(Debug, Clone)]
pub enum Constant {
    Number(f64),
    Bool(bool),
    String(String),
    Function(Function),
    BuiltInMethod(BuiltInMethod),
    Array(Rc<Vec<Constant>>),
    None,
}

impl Constant {
    pub fn get_pretty_type(&self) -> String {
        match self {
            Constant::Number(_) => "number".to_owned(),
            Constant::Bool(_) => "boolean".to_owned(),
            Constant::String(_) => "string".to_owned(),
            Constant::Function(f) => format!("fn <'{}' {}>", f.name, f.arity),
            Constant::None => "none".to_owned(),
            Constant::BuiltInMethod(f) => format!("fn <built-in '{}' {}>", f.name, f.arity),
            Constant::Array(arr) => format!("array <{}>", arr.len()),
        }
    }

    pub fn is_falsey(&self) -> bool {
        match &self {
            Self::Bool(value) => !value,
            Self::None => true,
            _ => false,
        }
    }

    pub fn get_string(&self) -> String {
        match self {
            Constant::Bool(b) => b.to_string(),
            Constant::Number(n) => n.to_string(),
            Constant::String(s) => s.to_owned(),
            Constant::None => "none".to_string(),
            Constant::Function(f) => format!("fn <'{}' {}>", f.name, f.arity),
            Constant::BuiltInMethod(f) => format!("fn <built-in '{}' {}>", f.name, f.arity),
            Constant::Array(arr) => format!("array <{}>", arr.len()),
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Constant::Bool(v) => write!(f, "{}", v),
            Constant::String(s) => write!(f, "\"{}\"", s),
            Constant::Number(n) => write!(f, "{}", n),
            Constant::None => write!(f, "none"),
            Constant::Function(func) => write!(f, "fn <'{}' {}>", func.name, func.arity),
            Constant::BuiltInMethod(func) => {
                write!(f, "fn <built-in '{}' {}>", func.name, func.arity)
            }
            Constant::Array(arr) => write!(f, "array <{}>", arr.len()),
        }
    }
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            Constant::Number(lhs) => {
                if let Constant::Number(rhs) = &other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Constant::Bool(lhs) => {
                if let Constant::Bool(rhs) = &other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Constant::String(lhs) => {
                if let Constant::String(rhs) = &other {
                    lhs == rhs
                } else {
                    false
                }
            }
            Constant::None => {
                matches!(other, Constant::None)
            }
            _ => false,
        }
    }
}

impl PartialOrd for Constant {
    fn ge(&self, other: &Self) -> bool {
        match &self {
            Constant::Number(lhs) => {
                if let Constant::Number(rhs) = &other {
                    lhs >= rhs
                } else {
                    panic!("Cannot compare non-numbers with '>=' operator")
                }
            }
            _ => panic!("Cannot compare non-numbers with '>=' operator"),
        }
    }

    fn le(&self, other: &Self) -> bool {
        match &self {
            Constant::Number(lhs) => {
                if let Constant::Number(rhs) = &other {
                    lhs <= rhs
                } else {
                    panic!("Cannot compare non-numbers with '>=' operator")
                }
            }
            _ => panic!("Cannot compare non-numbers with '>=' operator"),
        }
    }

    fn gt(&self, other: &Self) -> bool {
        match &self {
            Constant::Number(lhs) => {
                if let Constant::Number(rhs) = &other {
                    lhs > rhs
                } else {
                    panic!("Cannot compare non-numbers with '>=' operator")
                }
            }
            _ => panic!("Cannot compare non-numbers with '>=' operator"),
        }
    }

    fn lt(&self, other: &Self) -> bool {
        match &self {
            Constant::Number(lhs) => {
                if let Constant::Number(rhs) = &other {
                    lhs >= rhs
                } else {
                    panic!("Cannot compare non-numbers with '>=' operator")
                }
            }
            _ => panic!("Cannot compare non-numbers with '>=' operator"),
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (&self, &other) {
            (Constant::Number(lhs), Constant::Number(rhs)) => Some(lhs.partial_cmp(rhs).unwrap()),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct BuiltInMethod {
    pub name: String,
    pub func: Rc<dyn Fn(Vec<Constant>) -> Constant>,
    pub arity: u8,
}

impl BuiltInMethod {
    pub fn new(name: String, function: Rc<dyn Fn(Vec<Constant>) -> Constant>, arity: u8) -> Self {
        Self {
            name,
            func: function,
            arity,
        }
    }
}

impl std::fmt::Debug for BuiltInMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn <built-in '{}' {}", self.name, self.arity)
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
    Call(u8),
    Pop,
    Equal,
    NotEqual,
    Greater,
    Lesser,
    GreaterEq,
    LesserEq,
    Not,
    JumpIfFalse(u16),
    Jump(u16),
    IndexInto,
    ArrayLiteral(usize),
    Return,
}

impl Instruction {
    pub fn print_ins(&self, line: &usize, stack: Option<&VecDeque<Constant>>) {
        match &self {
            Instruction::Constant(Constant::Function(func)) => {
                println!("{:04}\tfn <'{}' {}>", line, func.name, func.arity);
                func.chunk.print_chunk();
            }
            Instruction::Call(index) => {
                if let Some(stack) = stack {
                    println!(
                        "{:04}\tCall({} at {})",
                        line,
                        &stack[(*index).into()],
                        index
                    );
                } else {
                    println!("{:04}\t{:?}", line, self);
                }
            }
            _ => println!("{:04}\t{:?}", line, self),
        }
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

    pub fn print_chunk(&self) {
        for (ins, line) in std::iter::zip(&self.code, &self.lines) {
            ins.print_ins(line, None);
        }
    }
}

impl std::ops::Index<usize> for Chunk {
    type Output = Instruction;

    fn index(&self, index: usize) -> &Self::Output {
        &self.code[index]
    }
}

impl std::ops::IndexMut<usize> for Chunk {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.code[index]
    }
}

#[derive(Debug, Clone)]
pub enum FunctionType {
    Script,
    Function,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: String,
    pub kind: FunctionType,
}

impl Function {
    pub fn new(arity: u8, chunk: Chunk, name: String, kind: FunctionType) -> Function {
        Function {
            arity,
            chunk,
            name,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function: Function,
    pub ip: usize,
    pub slot_offset: usize,
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
    pub fn new() -> Self {
        let locals = Vec::with_capacity(256);
        // locals.push(LocalVariable {
        // name: String::new(),
        // depth: 0,
        // });

        VariableManager {
            locals,
            scope_depth: 0,
        }
    }

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
            chunk.add_instruction(Instruction::SetGlobal(name.to_owned()), 0);
        } else {
            chunk.add_instruction(Instruction::GetGlobal(name.to_owned()), 0);
        }
    }

    pub fn resolve_local(&self, name: &str) -> Option<usize> {
        for (index, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(index + 1); // + 1 cuz we have a default value on the stack;
            }
        }
        None
    }
}
