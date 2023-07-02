use crate::{
    ast::{
        Assign, Binary, BinaryOp, Block, Call, ExprStmt, For, Function, FunctionArg, Grouping, If,
        Logical, LogicalOp, Node, Ret, Unary, UnaryOp, VarDecl,
    },
    tokenizer::{get_tok_len, get_tok_loc, TokenKind, Tokenizer},
};

macro_rules! matches {
    ($self: ident, $($tts:tt)*) => {
        if std::matches!($($tts)*) {
            $self.advance()?;
            true
        } else {
            false
        }
    };
}

macro_rules! consume {
    ($self: ident, $msg: expr, $($tts:tt)*) => {{
        if !matches!($self, $($tts)*) {
            return Err($self.error($msg, &$self.current))
        }
    }};
}

type ParseResult<T> = Result<T, String>;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current: TokenKind,
    source: &'a String,
    pub declarations: Vec<Node>,
}

impl<'a> Parser<'a> {
    pub fn new(mut tokenizer: Tokenizer<'a>, source: &'a String) -> ParseResult<Parser<'a>> {
        let current = tokenizer.next().unwrap()?;

        println!("{:#?}", current);
        Ok(Parser {
            tokenizer,
            current,
            source,
            declarations: Vec::new(),
        })
    }

    pub fn parse(&mut self) -> ParseResult<()> {
        let mut errors = Vec::new();
        while !self.is_at_end() {
            let declaration = self.declaration();

            match declaration {
                Ok(res) => {
                    if let Some(decl) = res {
                        let declaration = *decl;
                        self.declarations.push(declaration);
                    }
                }
                Err(e) => {
                    errors.push(format!("{}", e));
                    self.synchronize()?;
                }
            }
        }

        if !errors.is_empty() {
            let error = errors.join("\n");
            return Err(error);
        }
        Ok(())
    }

    fn synchronize(&mut self) -> ParseResult<()> {
        let mut previous = self.current.clone();
        self.advance()?;

        while !self.is_at_end() {
            if let TokenKind::ExprDelimiter(_, _) = previous {
                return Ok(());
            }
            match self.current {
                TokenKind::Func(_, _)
                | TokenKind::For(_, _)
                | TokenKind::If(_, _)
                | TokenKind::Ret(_, _)
                | TokenKind::End(_, _)
                | TokenKind::Else(_, _) => return Ok(()),
                _ => (),
            }
            previous = self.current.clone();
            self.advance()?;
        }

        Ok(())
    }

    fn error(&self, message: &str, token: &TokenKind) -> String {
        let mut lines = self.source.lines();
        let (line, column) = get_tok_loc(token).unwrap_or((1, 1));
        let source_line: &str = lines.nth(line - 1).unwrap();
        let src = source_line.trim_start();
        let offset = source_line.len() - src.len();

        let len = get_tok_len(token);

        format!(
            "{}:{} {}\n{}\n{}{}",
            line,
            column,
            message,
            src,
            " ".repeat(column - offset - len - 1),
            "~".repeat(len)
        )
    }

    fn declaration(&mut self) -> ParseResult<Option<Box<Node>>> {
        if matches!(self, self.current, TokenKind::Func(_, _)) {
            return Ok(Some(self.func_decl()?));
        }
        if matches!(self, self.current, TokenKind::Var(_, _)) {
            return Ok(Some(self.var_decl()?));
        }

        let stmt = self.statement()?;
        Ok(stmt)
    }

    fn statement(&mut self) -> ParseResult<Option<Box<Node>>> {
        if matches!(self, self.current, TokenKind::ExprDelimiter(_, _)) {
            return Ok(None);
        }

        let loc = get_tok_loc(&self.current);
        if matches!(self, self.current, TokenKind::Colon(_, _)) {
            let block = self.block()?;
            return Ok(Some(Block::new_node(block)));
        }
        if matches!(self, self.current, TokenKind::Ret(_, _)) {
            let stmt = self.ret_stmt(loc?)?;
            return Ok(Some(stmt));
        }
        if matches!(self, self.current, TokenKind::For(_, _)) {
            let stmt = self.for_stmt()?;
            return Ok(Some(stmt));
        }
        if matches!(self, self.current, TokenKind::If(_, _)) {
            let stmt = self.if_stmt()?;
            return Ok(Some(stmt));
        }

        let stmt = self.expr_stmt()?;
        Ok(Some(stmt))
    }

    fn var_decl(&mut self) -> ParseResult<Box<Node>> {
        let name;
        let name_loc;

        if let TokenKind::IdenLiteral(n, line, column) = &self.current {
            name = n.clone();
            name_loc = (*line, *column);
        } else {
            return Err(self.error("expected an identifier", &self.current));
        }

        self.advance()?;

        consume!(self, "expected '='", self.current, TokenKind::Equal(_, _));

        let value = self.expr()?;
        Ok(VarDecl::new_node(name, name_loc, value))
    }

    fn func_decl(&mut self) -> ParseResult<Box<Node>> {
        let name;
        let name_loc;
        if let TokenKind::IdenLiteral(literal, line, column) = &self.current {
            name = literal.clone();
            name_loc = (*line, *column);
        } else {
            return Err(self.error("expected an identifier", &self.current));
        }

        self.advance()?;

        let mut args = Vec::with_capacity(10);
        if let TokenKind::LeftParen(_, _) = &self.current {
            self.advance()?;
            loop {
                // stuff
                let arg_name;
                let arg_name_loc;
                if let TokenKind::IdenLiteral(literal, line, column) = &self.current {
                    arg_name = literal.clone();
                    arg_name_loc = (*line, *column);
                } else {
                    return Err(self.error("expected an identifier", &self.current));
                }

                args.push(FunctionArg::new(arg_name, arg_name_loc));
                self.advance()?;

                if !matches!(self, self.current, TokenKind::Comma(_, _)) {
                    break;
                }
            }
            consume!(
                self,
                "expected a ')'",
                self.current,
                TokenKind::RightParen(_, _)
            );
        }

        consume!(self, "expected a ':'", self.current, TokenKind::Colon(_, _));

        let body = self.block()?;
        Ok(Function::new_node(
            name,
            name_loc,
            args,
            Block::new_node(body),
        ))
    }

    fn if_stmt(&mut self) -> ParseResult<Box<Node>> {
        let cond = self.expr()?;
        consume!(self, "expected a ':'", self.current, TokenKind::Colon(_, _));

        let then_branch = self.block()?;
        let mut else_branch = None;
        if matches!(self, self.current, TokenKind::Else(_, _)) {
            consume!(self, "expected a ':'", self.current, TokenKind::Colon(_, _));
            else_branch = Some(Block::new_node(self.block()?));
        }

        Ok(If::new_node(
            cond,
            Block::new_node(then_branch),
            else_branch,
        ))
    }

    fn ret_stmt(&mut self, loc: (usize, usize)) -> ParseResult<Box<Node>> {
        let mut expr = None;
        if !std::matches!(self.current, TokenKind::ExprDelimiter(_, _)) {
            expr = Some(self.expr()?);
        }

        consume!(
            self,
            "expected a ';' or a new line",
            self.current,
            TokenKind::ExprDelimiter(_, _)
        );
        Ok(Ret::new_node(expr, loc))
    }

    fn for_stmt(&mut self) -> ParseResult<Box<Node>> {
        let name;
        let name_loc;
        if let TokenKind::IdenLiteral(n, line, column) = &self.current {
            name = n.clone();
            name_loc = (*line, *column);
            self.advance()?;
        } else {
            return Err(self.error("expected an identifier", &self.current));
        }

        consume!(self, "expected 'in'", self.current, TokenKind::In(_, _));
        let target = self.expr()?;

        consume!(self, "expected a ':'", self.current, TokenKind::Colon(_, _));
        let body = self.block()?;
        Ok(For::new_node(name, name_loc, target, Block::new_node(body)))
    }

    fn block(&mut self) -> ParseResult<Vec<Node>> {
        let mut errors = Vec::new();
        let mut statements: Vec<Node> = Vec::with_capacity(10);
        while !std::matches!(self.current, TokenKind::End(_, _)) && !self.is_at_end() {
            let declaration = self.declaration();
            match declaration {
                Ok(declaration) => {
                    if let Some(decl) = declaration {
                        let decl = *decl;
                        statements.push(decl);
                    }
                }
                Err(e) => {
                    errors.push(e);
                    self.synchronize()?;
                }
            }
        }

        if !errors.is_empty() {
            let error = errors.join("\n");
            return Err(error);
        }

        consume!(
            self,
            "Expected an 'end'",
            self.current,
            TokenKind::End(_, _)
        );

        Ok(statements)
    }

    fn expr_stmt(&mut self) -> ParseResult<Box<Node>> {
        let expr = self.expr()?;
        consume!(
            self,
            "Expected a ';' or a new line.",
            self.current,
            TokenKind::ExprDelimiter(_, _)
        );
        Ok(ExprStmt::new_node(expr))
    }

    fn expr(&mut self) -> ParseResult<Box<Node>> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Box<Node>> {
        let expr = self.or()?;
        if matches!(self, self.current, TokenKind::Equal(_, _)) {
            let value = self.assignment()?;

            match expr.as_ref() {
                Node::VarGet(name, line, column) => {
                    return Ok(Assign::new_node(name.to_string(), (*line, *column), value));
                }
                _ => return Err("Invalid target for assignment".to_string()),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.and()?;
        loop {
            #[allow(clippy::needless_late_init)]
            let lop;

            if matches!(self, self.current, TokenKind::Or(_, _)) {
                lop = LogicalOp::Or;
            } else {
                break;
            }

            let right = self.and()?;
            expr = Logical::new_node(expr, right, lop);
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.equality()?;
        loop {
            #[allow(clippy::needless_late_init)]
            let lop;

            if matches!(self, self.current, TokenKind::And(_, _)) {
                lop = LogicalOp::And;
            } else {
                break;
            }

            let right = self.equality()?;
            expr = Logical::new_node(expr, right, lop);
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.comparison()?;
        loop {
            let bop;

            if matches!(self, self.current, TokenKind::NotEqual(_, _)) {
                bop = BinaryOp::NotEqual;
            } else if matches!(self, self.current, TokenKind::EqualEqual(_, _)) {
                bop = BinaryOp::Equal;
            } else {
                break;
            }

            let right = self.comparison()?;
            expr = Binary::new_node(expr, right, bop);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.term()?;
        loop {
            let bop;

            if matches!(self, self.current, TokenKind::Greater(_, _)) {
                bop = BinaryOp::Greater;
            } else if matches!(self, self.current, TokenKind::GreaterEq(_, _)) {
                bop = BinaryOp::GreaterEq;
            } else if matches!(self, self.current, TokenKind::Less(_, _)) {
                bop = BinaryOp::Less;
            } else if matches!(self, self.current, TokenKind::LessEq(_, _)) {
                bop = BinaryOp::LessEq;
            } else {
                break;
            }

            let right = self.term()?;
            expr = Binary::new_node(expr, right, bop);
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.factor()?;
        loop {
            let bop;

            if matches!(self, self.current, TokenKind::Plus(_, _)) {
                bop = BinaryOp::Add;
            } else if matches!(self, self.current, TokenKind::Minus(_, _)) {
                bop = BinaryOp::Sub;
            } else {
                break;
            }

            let right = self.factor()?;
            expr = Binary::new_node(expr, right, bop);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.unary()?;
        loop {
            let bop;

            if matches!(self, self.current, TokenKind::Slash(_, _)) {
                bop = BinaryOp::Div;
            } else if matches!(self, self.current, TokenKind::Star(_, _)) {
                bop = BinaryOp::Mul;
            } else {
                break;
            }

            let right = self.unary()?;
            expr = Binary::new_node(expr, right, bop);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Box<Node>> {
        let mut uop = UnaryOp::None;
        let mut loc = (0, 0);

        if matches!(self, self.current, TokenKind::Bang(_, _)) {
            uop = UnaryOp::Not;
            loc = get_tok_loc(&self.current)?;
        } else if matches!(self, self.current, TokenKind::Minus(_, _)) {
            uop = UnaryOp::Negate;
            loc = get_tok_loc(&self.current)?;
        }

        if uop != UnaryOp::None {
            let expr = self.unary()?;
            return Ok(Unary::new_node(uop, loc, expr));
        }

        self.call()
    }

    fn call(&mut self) -> ParseResult<Box<Node>> {
        let mut expr = self.primary()?;
        loop {
            if matches!(self, self.current, TokenKind::LeftParen(_, _)) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Node>) -> ParseResult<Box<Node>> {
        let mut arguments = Vec::with_capacity(12);
        if !matches!(self, self.current, TokenKind::RightParen(_, _)) {
            loop {
                arguments.push(*self.expr()?);

                if !matches!(self, self.current, TokenKind::Comma(_, _)) {
                    break;
                }
            }
        }

        self.advance()?;

        Ok(Call::new_node(arguments, callee))
    }

    fn primary(&mut self) -> ParseResult<Box<Node>> {
        let node = match self.current.clone() {
            TokenKind::True(line, column) => Node::BoolLiteral(true, line, column),
            TokenKind::False(line, column) => Node::BoolLiteral(false, line, column),
            TokenKind::NumberLiteral(integer, line, column) => Node::Number(integer, line, column),
            TokenKind::StrLiteral(string, line, column) => {
                Node::StringLiteral(string, line, column)
            }
            TokenKind::IdenLiteral(ident, line, column) => Node::VarGet(ident, line, column),
            TokenKind::LeftParen(_, _) => {
                self.advance()?;
                let expr = self.expr()?;
                consume!(
                    self,
                    "Expected a ')'",
                    self.current,
                    TokenKind::RightParen(_, _)
                );
                return Ok(Grouping::new_node(expr));
            }
            _ => {
                return Err(self.error("unexpected token", &self.current));
            }
        };

        self.advance()?;
        Ok(Box::new(node))
    }

    fn advance(&mut self) -> ParseResult<()> {
        let next_token = self.tokenizer.next();

        println!("next token: {:#?}", next_token);
        self.current = next_token.unwrap_or(Ok(TokenKind::Eof))?;
        Ok(())
    }

    fn is_at_end(&mut self) -> bool {
        std::matches!(self.current, TokenKind::Eof)
    }
}
