use std::{cell::RefCell, fmt::Arguments, rc::Rc};

use crate::{
    ast::{BinaryOp, Node, UnaryOp},
    prelude::{Chunk, Constant, Function, FunctionType, Instruction, VariableManager},
};

#[derive(Default)]
pub struct Compiler {
    chunk: Chunk,
    var_manager: Rc<RefCell<VariableManager>>,
}

impl Compiler {
    pub fn new_with_manager(manager: Rc<RefCell<VariableManager>>) -> Self {
        Self {
            chunk: Chunk::default(),
            var_manager: manager,
        }
    }

    pub fn compile_node(&mut self, node: Node) {
        match node {
            Node::Number(number, line, _) => self.chunk.add_instruction(
                Instruction::Constant(Constant::Number(number.parse::<f64>().unwrap())),
                line,
            ),
            Node::NoneLiteral(line, _) => self
                .chunk
                .add_instruction(Instruction::Constant(Constant::None), line),
            Node::ArrayLiteral(values, line, _) => {
                let value_size = values.len();
                for val in values {
                    self.compile_node(val);
                }

                self.chunk
                    .add_instruction(Instruction::ArrayLiteral(value_size), line);
            }
            Node::Function(func) => {
                self.var_manager.borrow_mut().start_scope();
                let compiler = Compiler::new_with_manager(Rc::clone(&self.var_manager));
                for arg in &func.args {
                    self.var_manager
                        .borrow_mut()
                        .add_variable(&mut self.chunk, &arg.name);
                }

                let chunk = compiler.compile(vec![func.body]);

                self.var_manager.borrow_mut().end_scope(&mut self.chunk);
                self.chunk.add_instruction(
                    Instruction::Constant(Constant::Function(Function {
                        arity: func.args.len() as u8,
                        chunk,
                        name: func.name.to_owned(),
                        kind: FunctionType::Function,
                    })),
                    func.loc.0,
                );

                self.var_manager
                    .borrow_mut()
                    .add_variable(&mut self.chunk, &func.name);
            }
            Node::Block(block) => {
                self.var_manager.borrow_mut().start_scope();
                for decl in block.statements {
                    self.compile_node(decl);
                }
                self.var_manager.borrow_mut().end_scope(&mut self.chunk);
            }
            Node::Binary(binary) => {
                self.compile_node(*binary.lhs);
                self.compile_node(*binary.rhs);

                let instruction = match &binary.op {
                    BinaryOp::Add => Instruction::Add,
                    BinaryOp::Sub => Instruction::Subtract,
                    BinaryOp::Mul => Instruction::Multiply,
                    BinaryOp::Div => Instruction::Divide,
                    BinaryOp::Equal => Instruction::Equal,
                    BinaryOp::NotEqual => Instruction::NotEqual,
                    BinaryOp::Greater => Instruction::Greater,
                    BinaryOp::GreaterEq => Instruction::GreaterEq,
                    BinaryOp::Less => Instruction::Lesser,
                    BinaryOp::LessEq => Instruction::LesserEq,
                };

                self.chunk.add_instruction(instruction, 1); // TODO: fix line location
            }
            Node::ExprStmt(stmt) => {
                self.compile_node(*stmt.expr);
                self.chunk.add_instruction(Instruction::Pop, 1); // TODO: fix line location
            }
            Node::Call(call) => {
                self.compile_node(*call.callee);

                let arg_count = call.args.len();
                for arg in call.args {
                    self.compile_node(arg);
                }

                // TODO: fix line location
                self.chunk
                    .add_instruction(Instruction::Call(arg_count as u8), 1);
            }
            Node::Ret(ret) => {
                if let Some(value) = ret.value {
                    self.compile_node(*value);
                } else {
                    self.chunk
                        .add_instruction(Instruction::Constant(Constant::None), 1);
                    // TODO: fix location
                }

                self.chunk.add_instruction(Instruction::Return, 1); // TODO: fix location
            }
            Node::VarGet(name, _, _) => {
                self.var_manager
                    .borrow_mut()
                    .named_variable(&name, false, &mut self.chunk);
            }
            Node::VarDecl(decl) => {
                self.compile_node(*decl.value);
                self.var_manager
                    .borrow_mut()
                    .add_variable(&mut self.chunk, &decl.name);
            }
            Node::StringLiteral(s, line, _) => self
                .chunk
                .add_instruction(Instruction::Constant(Constant::String(s)), line),
            Node::Subscript(susbcript) => {
                self.compile_node(*susbcript.value);
                self.compile_node(*susbcript.index);
                self.chunk.add_instruction(Instruction::IndexInto, 0);
            }
            Node::BoolLiteral(value, line, _) => self
                .chunk
                .add_instruction(Instruction::Constant(Constant::Bool(value)), line),
            Node::Grouping(group) => self.compile_node(*group.expr),
            Node::Unary(unary) => {
                self.compile_node(*unary.expr);
                match unary.op {
                    UnaryOp::Negate => {
                        self.chunk
                            .add_instruction(Instruction::Negate, unary.op_loc.0);
                    }
                    UnaryOp::Not => {
                        self.chunk.add_instruction(Instruction::Not, unary.op_loc.0);
                    }
                    _ => unreachable!(),
                }
            }
            Node::Logical(_) => todo!(),
            Node::Assign(_) => todo!(),
            Node::For(for_stmt) => {
                unimplemented!();
                let loop_start = self.chunk.code.len();
                let exit_jump = -1;

                self.compile_node(*for_stmt.body);
                self.emit_loop(loop_start);
            },
            Node::If(if_stmt) => {
                self.compile_node(*if_stmt.condition);

                // TODO: actual lines
                let then_jump = self.chunk.emit_jump(Instruction::JumpIfFalse(0), 1);
                self.chunk.add_instruction(Instruction::Pop, 1);
                self.compile_node(*if_stmt.then_block);

                let else_jump = self.chunk.emit_jump(Instruction::Jump(0), 1);
                self.patch_jump(then_jump);
                self.chunk.add_instruction(Instruction::Pop, 1);

                if let Some(else_cond) = if_stmt.else_block {
                    self.compile_node(*else_cond);
                }
                self.patch_jump(else_jump);
            } // _ => {
              //     println!("{node:#?}");
              //     unimplemented!()
              // }
        }
    }

    pub fn compile(mut self, declarations: Vec<Box<Node>>) -> Chunk {
        for decl in declarations {
            self.compile_node(*decl);
        }

        let last = self.chunk.code.last();

        match last {
            Some(Instruction::Return) => (),
            _ => {
                self.chunk
                    .add_instruction(Instruction::Constant(Constant::None), 1);
                self.chunk.add_instruction(Instruction::Return, 1);
            }
        };
        self.chunk
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset;
        self.chunk.code[offset] = Instruction::JumpIfFalse(jump as u16);
    }

    fn emit_loop(&mut self, offset: usize) {
        
    }

    pub fn compile_non_boxed(mut self, declarations: Vec<Node>) -> Chunk {
        for decl in declarations {
            self.compile_node(decl);
        }
        self.chunk
    }
}
