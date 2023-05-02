use std::collections::{HashMap, VecDeque};

use log::debug;

use crate::{
    bytecode::{CallFrame, Constant, Function, Instruction},
    prelude::BuiltInMethod,
};

#[derive(Debug, Clone)]
pub struct VirtualMachine {
    stack: VecDeque<Constant>,
    globals: HashMap<String, Constant>,
    frames: Vec<CallFrame>,
}

impl VirtualMachine {
    pub fn new(script: Function) -> Self {
        let mut frames = Vec::with_capacity(8);

        frames.push(CallFrame {
            ip: 0,
            function: script,
            slot_offset: 0,
        });

        VirtualMachine {
            frames,
            stack: VecDeque::with_capacity(256),
            globals: HashMap::with_capacity(32),
        }
    }

    pub fn define_built_in_fn(&mut self, method: BuiltInMethod) {
        self.globals
            .insert(method.name.to_owned(), Constant::BuiltInMethod(method));
    }

    pub fn print_stack(&self) {
        if !self.stack.is_empty() {
            print!("stack: ");
            for constant in &self.stack {
                if let Constant::Function(func) = constant {
                    print!("fn '{}'", func.name);
                } else {
                    print!("[{constant:?}] ");
                }
            }
            println!()
        }
    }

    fn error(&self, message: &str) -> ! {
        let frame = self.frames.last().unwrap();
        self.error_ip(message, frame.ip)
    }

    fn error_ip(&self, message: &str, ip: usize) -> ! {
        let frame = self.frames.last().unwrap();
        let ins = &frame.function.chunk[ip];
        let line = &frame.function.chunk.lines[ip];

        panic!(
            "VMerror: {message} at line '{}' on instruction '{:?}'",
            line, ins
        )
    }

    fn call_value(&mut self, constant: Constant, arg_count: u8) -> bool {
        match constant {
            Constant::Function(func) => {
                if func.arity != arg_count {
                    self.error(&format!(
                        "Function '{}' accepts {} arguments but {} were provided.",
                        func.name, func.arity, arg_count
                    ));
                }

                let frame = CallFrame {
                    function: func,
                    ip: 0,
                    slot_offset: self.stack.len() - arg_count as usize - 1,
                };

                self.frames.push(frame);
                true
            }
            Constant::BuiltInMethod(func) => {
                if func.arity != 0 && func.arity != arg_count {
                    self.error(&format!(
                        "Function '{}' accepts {} arguments but {} were provided.",
                        func.name, func.arity, arg_count
                    ));
                }

                let removed = self
                    .stack
                    .range(self.stack.len() - arg_count as usize..)
                    .map(|c| c.to_owned())
                    .collect::<Vec<Constant>>();

                println!(
                    "Built-in method called: {} at address {:?}",
                    func.name, func.function
                );

                let function = func.function;
                let result = function(removed);

                println!("Stack before: {:#?}", self.stack);
                self.stack
                    .truncate(self.stack.len() - arg_count as usize + 1);
                println!("Stack after: {:#?}", self.stack);
                self.stack.push_back(result);
                true
            }
            _ => false,
        }
    }

    fn peek_back(&self) -> &Constant {
        self.stack.back().expect("VMError: failed to peek_back")
    }

    fn peek(&self, distance: usize) -> &Constant {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .expect("Failed to peek");
    }

    pub fn interpret(&mut self) {
        loop {
            let mut frame = self.frames.last().unwrap();
            let ins = &frame.function.chunk[frame.ip];
            let line = &frame.function.chunk.lines[frame.ip];

            self.print_stack();
            ins.print_ins(line, Some(&self.stack));

            match ins {
                Instruction::Constant(constant) => {
                    self.stack.push_back(constant.clone());
                }
                Instruction::Add => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => number,
                        _ => self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )),
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )),
                    };

                    self.stack.push_back(Constant::Number(lhs + rhs));
                }
                Instruction::Subtract => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => number,
                        _ => self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )),
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )),
                    };

                    self.stack.push_back(Constant::Number(lhs - rhs));
                }
                Instruction::Divide => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => {
                            if number == 0.0 {
                                self.error("cannot divide a number by zero")
                            }
                            number
                        }
                        _ => self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )),
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )),
                    };

                    self.stack.push_back(Constant::Number(lhs / rhs));
                }
                Instruction::Multiply => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => {
                            if number == 0.0 {
                                self.error("cannot divide a number by zero")
                            }
                            number
                        }
                        _ => self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )),
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )),
                    };

                    self.stack.push_back(Constant::Number(lhs * rhs));
                }
                Instruction::GetGlobal(name) => {
                    if let Some(global) = self.globals.get(name) {
                        self.stack.push_back(global.clone());
                    } else {
                        self.error(&format!("no global with name '{}' exists", name))
                    }
                }
                Instruction::DefineGlobal(name) => {
                    let value = self.peek_back().clone();

                    self.globals.insert(name.to_owned(), value.clone());
                    self.stack.pop_back(); // we pop the value that we `peek_back()`'d
                }
                Instruction::SetGlobal(name) => {
                    let value = self.peek_back().clone();
                    self.globals.insert(name.to_owned(), value);
                    // we do not pop the value because `(x = 3) + 1` should be a valid expression
                    // where 3 will be on the stack therefore summing up with 1 and giving the result.
                }
                Instruction::GetLocal(index) => {
                    let index = self.frames.last().unwrap().slot_offset + *index;

                    self.stack.push_back(
                        self.stack
                            .get(index)
                            .unwrap_or_else(|| self.error("no such local variable in the scope"))
                            .to_owned(),
                    );
                }
                Instruction::SetLocal(index) => {
                    let index = self.frames.last().unwrap().slot_offset + *index;

                    let value = self
                        .stack
                        .pop_back()
                        .unwrap_or_else(|| self.error("no value for local variable to set"));
                    let local = self.stack.get_mut(index);

                    if let Some(local) = local {
                        *local = value;
                    } else {
                        self.error("no such local variable in the scope");
                    }
                }
                Instruction::Pop => {
                    self.stack.pop_back();
                }
                Instruction::Call(arg_count) => {
                    self.stack.len();
                    let function = self.peek(*arg_count as usize).to_owned();
                    let value = self.call_value(function, *arg_count);
                    if !value {
                        unimplemented!()
                    }
                    continue;
                }
                Instruction::JumpIfFalse(offset) => {
                    if self.peek(0).is_falsey() {
                        self.frames.last_mut().unwrap().ip += *offset as usize;
                    }
                }
                Instruction::Jump(offset) => {
                    self.frames.last_mut().unwrap().ip += *offset as usize;
                }
                Instruction::Equal => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(a == b));
                }
                Instruction::NotEqual => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(a != b));
                }
                Instruction::Greater => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(a > b));
                }
                Instruction::GreaterEq => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(a >= b));
                }
                Instruction::Lesser => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(a < b));
                }
                Instruction::LesserEq => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(a <= b));
                }
                Instruction::Not => {
                    let value = self.stack.pop_back().unwrap();

                    self.stack.push_back(Constant::Bool(value.is_falsey()));
                }
                Instruction::Return => {
                    // self.stack.truncate(self.frames.last().unwrap().slot_offset);
                    let ret_val = self.stack.pop_back().unwrap();

                    let offset = self.frames.last().unwrap().slot_offset;
                    self.frames.pop();

                    if self.frames.is_empty() {
                        return;
                    }

                    println!("truncate stack?");
                    
                    self.stack.truncate(offset);
                    self.stack.push_back(ret_val);
                }
                _ => unimplemented!(),
            }

            self.frames.last_mut().unwrap().ip += 1;
        }
    }
}
