#[derive(Debug, Clone)]
pub struct FunctionArg {
    pub name: String,
    pub name_loc: (usize, usize),
}

impl FunctionArg {
    pub fn new(name: String, name_loc: (usize, usize)) -> FunctionArg {
        FunctionArg { name, name_loc }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Number(String, usize, usize),
    StringLiteral(String, usize, usize),
    BoolLiteral(bool, usize, usize),
    // ArrayLiteral(Vec<Node>, usize, usize),
    VarGet(String, usize, usize),
    Binary(Binary),
    Function(Function),
    VarDecl(VarDecl),
    Grouping(Grouping),
    Unary(Unary),
    Logical(Logical),
    Assign(Assign),
    For(For),
    If(If),
    // Use(Use),
    Ret(Ret),
    Block(Block),
    ExprStmt(ExprStmt),
    Call(Call),
}

#[derive(PartialEq, Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Equal,
    NotEqual,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Greater => write!(f, ">"),
            Self::GreaterEq => write!(f, ">="),
            Self::Less => write!(f, "<"),
            Self::LessEq => write!(f, "<="),
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

impl std::fmt::Display for LogicalOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum UnaryOp {
    Not,
    Negate,
    None,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Not => write!(f, "!"),
            Self::Negate => write!(f, "-"),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
    pub op: BinaryOp,
}

impl Binary {
    pub fn new_node(lhs: Box<Node>, rhs: Box<Node>, op: BinaryOp) -> Box<Node> {
        Box::new(Node::Binary(Binary { lhs, rhs, op }))
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub loc: (usize, usize),
    pub args: Vec<FunctionArg>,
    pub body: Box<Node>,
}

impl Function {
    pub fn new_node(
        name: String,
        loc: (usize, usize),
        args: Vec<FunctionArg>,
        body: Box<Node>,
    ) -> Box<Node> {
        Box::new(Node::Function(Function {
            name,
            loc,
            args,
            body,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub args: Vec<Node>,
    pub callee: Box<Node>,
}

impl Call {
    pub fn new_node(args: Vec<Node>, callee: Box<Node>) -> Box<Node> {
        Box::new(Node::Call(Call { args, callee }))
    }
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub name_loc: (usize, usize),
    pub value: Box<Node>,
}

impl VarDecl {
    pub fn new_node(name: String, name_loc: (usize, usize), value: Box<Node>) -> Box<Node> {
        Box::new(Node::VarDecl(VarDecl {
            name,
            name_loc,
            value,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expr: Box<Node>,
}

impl Grouping {
    pub fn new_node(expr: Box<Node>) -> Box<Node> {
        Box::new(Node::Grouping(Grouping { expr }))
    }
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub op: UnaryOp,
    pub op_loc: (usize, usize),
    pub expr: Box<Node>,
}

impl Unary {
    pub fn new_node(op: UnaryOp, op_loc: (usize, usize), expr: Box<Node>) -> Box<Node> {
        Box::new(Node::Unary(Unary { op, op_loc, expr }))
    }
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
    pub op: LogicalOp,
}

impl Logical {
    pub fn new_node(lhs: Box<Node>, rhs: Box<Node>, op: LogicalOp) -> Box<Node> {
        Box::new(Node::Logical(Logical { lhs, rhs, op }))
    }
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: String,
    pub name_loc: (usize, usize),
    pub value: Box<Node>,
}

impl Assign {
    pub fn new_node(name: String, name_loc: (usize, usize), value: Box<Node>) -> Box<Node> {
        Box::new(Node::Assign(Assign {
            name,
            name_loc,
            value,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct For {
    pub name: String,
    pub name_loc: (usize, usize),
    pub target: Box<Node>,
    pub body: Box<Node>,
}

impl For {
    pub fn new_node(
        name: String,
        name_loc: (usize, usize),
        target: Box<Node>,
        body: Box<Node>,
    ) -> Box<Node> {
        Box::new(Node::For(For {
            name,
            name_loc,
            target,
            body,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Node>,
    pub then_block: Box<Node>,
    pub else_block: Option<Box<Node>>,
}

impl If {
    pub fn new_node(
        condition: Box<Node>,
        then_block: Box<Node>,
        else_block: Option<Box<Node>>,
    ) -> Box<Node> {
        Box::new(Node::If(If {
            condition,
            then_block,
            else_block,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Ret {
    pub value: Option<Box<Node>>,
    pub loc: (usize, usize),
}

impl Ret {
    pub fn new_node(value: Option<Box<Node>>, loc: (usize, usize)) -> Box<Node> {
        Box::new(Node::Ret(Ret { value, loc }))
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Node>,
}

impl Block {
    pub fn new_node(statements: Vec<Node>) -> Box<Node> {
        Box::new(Node::Block(Block { statements }))
    }
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Box<Node>,
}

impl ExprStmt {
    pub fn new_node(expr: Box<Node>) -> Box<Node> {
        Box::new(Node::ExprStmt(ExprStmt { expr }))
    }
}
