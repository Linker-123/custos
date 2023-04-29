use std::{collections::VecDeque, rc::Rc};

use crate::bytecode::{Chunk, Constant, Instruction};

#[derive(Debug, Clone)]
pub struct VirtualMachine {
    stack: VecDeque<Constant>,
    chunk: Rc<Chunk>,
    ip: usize,
}

impl VirtualMachine {
    pub fn new(chunk: Chunk) -> Self {
        VirtualMachine {
            stack: VecDeque::with_capacity(256),
            chunk: Rc::new(chunk),
            ip: 0,
        }
    }

    pub fn print_stack(&self) {
        if !self.stack.is_empty() {
            for constant in &self.stack {
                print!("[{constant:?}] ");
            }
            println!()
        }
    }

    fn error(&self, message: &str) -> ! {
        panic!(
            "VMerror: {message} at line '{}' on instruction '{:?}'",
            &self.chunk.lines[self.ip], &self.chunk[self.ip]
        )
    }

    pub fn interpret(&mut self) {
        loop {
            let ins = &self.chunk[self.ip];
            let line = &self.chunk.lines[self.ip];

            self.print_stack();
            ins.print_ins(*line);

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
                Instruction::Return => return,
                _ => unimplemented!(),
            }

            self.ip += 1;
        }
    }
}
