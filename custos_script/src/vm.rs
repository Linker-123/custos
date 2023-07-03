use crate::{
    bytecode::{CallFrame, Constant, Function, Instruction},
    prelude::BuiltInMethod,
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

pub enum CallResult {
    Ok,
    OkNative,
    Err,
}

#[derive(Debug)]
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

    fn error(&self, message: &str) -> String {
        let frame = self.frames.last().unwrap();
        self.error_ip(message, frame.ip)
    }

    fn error_ip(&self, message: &str, ip: usize) -> String {
        let frame = self.frames.last().unwrap();
        let ins = &frame.function.chunk[ip];
        let line = &frame.function.chunk.lines[ip];

        format!(
            "VMerror: {message} at line '{}' on instruction '{:?}'",
            line, ins
        )
    }

    fn call_value(&mut self, constant: Constant, arg_count: u8) -> CallResult {
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
                CallResult::Ok
            }
            Constant::BuiltInMethod(func) => {
                if func.arity != 0 && func.arity != arg_count {
                    self.error(&format!(
                        "Function '{}' accepts {} arguments but {} were provided.",
                        func.name, func.arity, arg_count
                    ));
                }

                println!("Arg count: {}", arg_count);

                let removed = self
                    .stack
                    .range(self.stack.len() - arg_count as usize..)
                    .map(|c| c.to_owned())
                    .collect::<Vec<Constant>>();

                println!("Built-in method called: {}", func.name);

                // let result = func.func(removed);
                let callable = func.func;
                let result = callable(removed);

                // println!(
                //     "result: {:#?}, stack before: {}, stack after: {}",
                //     result,
                //     self.stack.len(),
                //     self.stack.len() - arg_count as usize
                // );
                self.stack
                    .truncate(self.stack.len() - arg_count as usize - 1);
                self.stack.push_back(result);
                CallResult::OkNative
            }
            _ => CallResult::Err,
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

    pub fn interpret(&mut self) -> Option<String> {
        loop {
            let frame = self.frames.last().unwrap();
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

                    if matches!(a, Constant::String(_)) || matches!(b, Constant::String(_)) {
                        let mut a = a.get_string();
                        let b = b.get_string();

                        a.push_str(&b);

                        self.stack.push_back(Constant::String(a));
                    } else {
                        let rhs = match b {
                            Constant::Number(number) => number,
                            _ => {
                                return Some(self.error(&format!(
                                "cannot add two non-numbers, right-hand side is not a number but a {}",
                                b.get_pretty_type()
                            )))
                            }
                        };

                        let lhs = match a {
                            Constant::Number(number) => number,
                            _ => {
                                return Some(self.error(&format!(
                                "cannot add two non-numbers, left-hand side is not a number but a {}",
                                a.get_pretty_type()
                            )))
                            }
                        };

                        self.stack.push_back(Constant::Number(lhs + rhs));
                    }
                }
                Instruction::Subtract => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => number,
                        _ => {
                            return Some(self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )))
                        }
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => {
                            return Some(self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )))
                        }
                    };

                    self.stack.push_back(Constant::Number(lhs - rhs));
                }
                Instruction::Divide => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => {
                            if number == 0.0 {
                                return Some(self.error("cannot divide a number by zero"));
                            }
                            number
                        }
                        _ => {
                            return Some(self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )))
                        }
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => {
                            return Some(self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )))
                        }
                    };

                    self.stack.push_back(Constant::Number(lhs / rhs));
                }
                Instruction::Multiply => {
                    let b = self.stack.pop_back().unwrap();
                    let a = self.stack.pop_back().unwrap();

                    let rhs = match b {
                        Constant::Number(number) => {
                            if number == 0.0 {
                                return Some(self.error("cannot divide a number by zero"));
                            }
                            number
                        }
                        _ => {
                            return Some(self.error(&format!(
                            "cannot add two non-numbers, right-hand side is not a number but a {}",
                            b.get_pretty_type()
                        )))
                        }
                    };

                    let lhs = match a {
                        Constant::Number(number) => number,
                        _ => {
                            return Some(self.error(&format!(
                            "cannot add two non-numbers, left-hand side is not a number but a {}",
                            a.get_pretty_type()
                        )))
                        }
                    };

                    self.stack.push_back(Constant::Number(lhs * rhs));
                }
                Instruction::GetGlobal(name) => {
                    if let Some(global) = self.globals.get(name) {
                        self.stack.push_back(global.clone());
                    } else {
                        return Some(self.error(&format!("no global with name '{}' exists", name)));
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

                    self.stack.push_back(match self.stack.get(index) {
                        Some(d) => d.to_owned(),
                        None => return Some(self.error("no such local variable in the scope")),
                    });
                }
                Instruction::SetLocal(index) => {
                    let index = self.frames.last().unwrap().slot_offset + *index;

                    let value = match self.stack.pop_back() {
                        Some(d) => d,
                        None => {
                            return Some(self.error("no value for local variable to set"));
                        }
                    };
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
                    let function = self.peek(*arg_count as usize).to_owned();
                    let value = self.call_value(function, *arg_count);

                    match value {
                        CallResult::Err => return Some(self.error("Cant call a non-function")),
                        CallResult::OkNative => {
                            // because native functions dont have RETURN
                            self.frames.last_mut().unwrap().ip += 1;
                        }
                        _ => (),
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
                Instruction::IndexInto => {
                    let index = self.stack.pop_back().unwrap();
                    let array_value = self.stack.pop_back().unwrap();

                    let index = match index {
                        Constant::Number(n) => n as usize,
                        _ => return Some(self.error("Invalid index")),
                    };

                    if let Constant::String(s) = array_value {
                        let character = s.chars().nth(index);
                        self.stack.push_back(match character {
                            Some(c) => Constant::String(String::from(c)),
                            None => Constant::None,
                        });
                    } else if let Constant::Array(array) = array_value {
                        let element = array.get(index);

                        self.stack.push_back(match element {
                            Some(v) => v.to_owned(),
                            None => Constant::None,
                        });
                    } else {
                        return Some(self.error(&format!(
                            "Can only index into a string or array, got: {}",
                            array_value.get_pretty_type()
                        )));
                    }

                    // println!("Indexing: {:?}, into array: {:?}", index, array_value);
                }
                Instruction::ArrayLiteral(offset) => {
                    let mut values = Vec::new();

                    for _ in 0..*offset {
                        values.push(self.stack.pop_back().unwrap());
                    }

                    values.reverse();
                    self.stack.push_back(Constant::Array(Rc::new(values)));
                }
                Instruction::Return => {
                    // self.stack.truncate(self.frames.last().unwrap().slot_offset);
                    let ret_val = self.stack.pop_back().unwrap();

                    let offset = self.frames.last().unwrap().slot_offset;
                    self.frames.pop();

                    if self.frames.is_empty() {
                        return None;
                    }

                    self.stack.truncate(offset);
                    self.stack.push_back(ret_val);
                }
            }

            self.frames.last_mut().unwrap().ip += 1;
        }
    }
}
