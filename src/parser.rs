use crate::ast::Node;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Dot,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Lt,
    Gt,
    Assign,
    Arrow,    // ->
    FatArrow, // =>
    Amp,      // &
    Shl,      // <<
    Shr,      // >>
    KeywordLet,
    KeywordIf,
    KeywordElse,
    KeywordWhile,
    KeywordFn,
    KeywordReturn,
    KeywordImport,
    BuiltinNull,
    EOF,
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    pub line: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos] as char)
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.peek_char() {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_ascii_whitespace() {
                self.advance();
            } else if c == '/'
                && self.pos + 1 < self.input.len()
                && self.input[self.pos + 1] as char == '/'
            {
                while let Some(c2) = self.peek_char() {
                    if c2 == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Token::EOF;
        }

        let c = self.peek_char().unwrap();

        if c.is_ascii_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    s.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            return match s.as_str() {
                "let" => Token::KeywordLet,
                "if" => Token::KeywordIf,
                "else" => Token::KeywordElse,
                "while" => Token::KeywordWhile,
                "fn" => Token::KeywordFn,
                "return" => Token::KeywordReturn,
                "import" => Token::KeywordImport,
                "null" => Token::BuiltinNull,
                _ => Token::Ident(s),
            };
        }

        if c.is_ascii_digit() {
            let mut s = String::new();
            let mut is_float = false;
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    s.push(ch);
                    self.advance();
                } else if ch == '.' {
                    let next_ch = if self.pos + 1 < self.input.len() {
                        self.input[self.pos + 1] as char
                    } else {
                        ' '
                    };
                    if next_ch.is_ascii_alphabetic() {
                        // Prevent eating `.prop`
                        break;
                    }
                    is_float = true;
                    s.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            if is_float {
                return Token::Float(s.parse().unwrap());
            } else {
                return Token::Int(s.parse().unwrap());
            }
        }

        if c == '"' {
            self.advance();
            let mut s = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == '"' {
                    self.advance();
                    break;
                }
                s.push(ch);
                self.advance();
            }
            return Token::Str(s);
        }

        self.advance();
        let next_c = self.peek_char().unwrap_or(' ');

        match c {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semi,
            '.' => Token::Dot,
            '+' => Token::Plus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '&' => Token::Amp,
            '-' => {
                if next_c == '>' {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '=' => {
                if next_c == '=' {
                    self.advance();
                    Token::EqEq
                } else if next_c == '>' {
                    self.advance();
                    Token::FatArrow
                } else {
                    Token::Assign
                }
            }
            '<' => {
                if next_c == '<' {
                    self.advance();
                    Token::Shl
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if next_c == '>' {
                    self.advance();
                    Token::Shr
                } else {
                    Token::Gt
                }
            }
            _ => {
                let escaped_hint = format!("Unexpected char '{}'", c).replace("\"", "\\\"");
                let json = format!(
                    r#"{{"diagnostic": {{"line": {}, "col": {}, "hint": "{}"}}}}"#,
                    self.line, self.col, escaped_hint
                );
                panic!("{}", json);
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<(Token, usize, usize)>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let line = lexer.line;
            let col = lexer.col;
            let t = lexer.next_token();
            tokens.push((t.clone(), line, col));
            if t == Token::EOF {
                break;
            }
        }
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].0
    }

    fn peek_pos(&self) -> (usize, usize) {
        (self.tokens[self.pos].1, self.tokens[self.pos].2)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].0.clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn diagnostic_panic(&self, hint: &str) -> ! {
        let (line, col) = self.peek_pos();
        let escaped_hint = hint.replace("\"", "\\\"");
        let json = format!(
            r#"{{"diagnostic": {{"line": {}, "col": {}, "hint": "{}"}}}}"#,
            line, col, escaped_hint
        );
        panic!("{}", json);
    }

    fn expect(&mut self, expected: Token) {
        let (line, col) = self.peek_pos();
        let t = self.advance();
        if t != expected {
            let escaped_hint =
                format!("Expected {:?}, found {:?}", expected, t).replace("\"", "\\\"");
            let json = format!(
                r#"{{"diagnostic": {{"line": {}, "col": {}, "hint": "{}"}}}}"#,
                line, col, escaped_hint
            );
            panic!("{}", json);
        }
    }

    pub fn parse(&mut self) -> Node {
        let mut statements = Vec::new();
        while *self.peek() != Token::EOF {
            statements.push(self.parse_statement());
        }
        Node::Block(statements)
    }

    fn parse_statement(&mut self) -> Node {
        match self.peek() {
            Token::KeywordLet => {
                self.advance();
                let ident = match self.advance() {
                    Token::Ident(name) => name,
                    _ => self.diagnostic_panic("Expected identifier after let"),
                };
                self.expect(Token::Assign);
                let expr = self.parse_expression();
                self.expect(Token::Semi);
                Node::Assign(ident, Box::new(expr))
            }
            Token::KeywordIf => {
                self.advance();
                self.expect(Token::LParen);
                let cond = self.parse_expression();
                self.expect(Token::RParen);
                let then_branch = self.parse_block();
                let mut else_branch = None;
                if *self.peek() == Token::KeywordElse {
                    self.advance();
                    else_branch = Some(Box::new(self.parse_block()));
                }
                Node::If(Box::new(cond), Box::new(then_branch), else_branch)
            }
            Token::KeywordWhile => {
                self.advance();
                self.expect(Token::LParen);
                let cond = self.parse_expression();
                self.expect(Token::RParen);
                let body = self.parse_block();
                Node::While(Box::new(cond), Box::new(body))
            }
            Token::KeywordFn => {
                self.advance();
                let name = match self.advance() {
                    Token::Ident(name) => name,
                    _ => self.diagnostic_panic("Expected function name"),
                };
                self.expect(Token::LParen);
                let mut args = Vec::new();
                while *self.peek() != Token::RParen {
                    if let Token::Ident(arg) = self.advance() {
                        args.push(arg);
                    }
                    if *self.peek() == Token::Comma {
                        self.advance();
                    }
                }
                self.expect(Token::RParen);
                let body = self.parse_block();
                Node::FnDef(name, args, Box::new(body))
            }
            Token::KeywordReturn => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(Token::Semi);
                Node::Return(Box::new(expr))
            }
            Token::LBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expression();

                // Check for -> { block } which is If(expr, Block, None)
                if *self.peek() == Token::Arrow {
                    self.advance();
                    let block = self.parse_block();
                    return Node::If(Box::new(expr), Box::new(block), None);
                }

                // Check for fat arrow => { block } for async callbacks (Fetch)
                if *self.peek() == Token::FatArrow {
                    self.advance();
                    let callback = self.parse_block();

                    if let Node::Call(name, args) = expr {
                        if name == "Fetch" && args.len() == 2 {
                            let method = if let Node::StringLiteral(s) = &args[0] {
                                s.clone()
                            } else {
                                self.diagnostic_panic("Fetch expects Method as string")
                            };
                            let url = if let Node::StringLiteral(s) = &args[1] {
                                s.clone()
                            } else {
                                self.diagnostic_panic("Fetch expects URL as string")
                            };
                            return Node::Fetch {
                                method,
                                url,
                                callback: Box::new(callback),
                            };
                        }
                    }
                    self.diagnostic_panic(
                        "FatArrow '=>' can only be used with Fetch(method, url) calls",
                    );
                }

                if *self.peek() == Token::Semi {
                    self.advance(); // consume semi
                }
                expr
            }
        }
    }

    fn parse_block(&mut self) -> Node {
        self.expect(Token::LBrace);
        let mut stmts = Vec::new();
        while *self.peek() != Token::RBrace && *self.peek() != Token::EOF {
            stmts.push(self.parse_statement());
        }
        self.expect(Token::RBrace);
        Node::Block(stmts)
    }

    fn parse_expression(&mut self) -> Node {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Node {
        let left = self.parse_comparison();
        if *self.peek() == Token::Assign {
            self.advance();
            let right = self.parse_expression(); // right-associative
            match left {
                Node::Identifier(name) => Node::Assign(name, Box::new(right)),
                Node::ArrayGet(arr, index) => Node::ArraySet(arr, index, Box::new(right)),
                Node::MapGet(map, key) => Node::MapSet(map, key, Box::new(right)),
                Node::PropertyGet(obj, prop) => Node::PropertySet(obj, prop, Box::new(right)),
                Node::Index(container, idx) => Node::ArraySet(container, idx, Box::new(right)), // Fallback mapping
                _ => self.diagnostic_panic("Invalid assignment target"),
            }
        } else {
            left
        }
    }

    fn parse_comparison(&mut self) -> Node {
        let mut node = self.parse_term();
        loop {
            match self.peek() {
                Token::EqEq => {
                    self.advance();
                    node = Node::Eq(Box::new(node), Box::new(self.parse_term()));
                }
                Token::Lt => {
                    self.advance();
                    node = Node::Lt(Box::new(node), Box::new(self.parse_term()));
                }
                Token::Gt => {
                    self.advance();
                    node = Node::Gt(Box::new(node), Box::new(self.parse_term()));
                }
                _ => break,
            }
        }
        node
    }

    fn parse_term(&mut self) -> Node {
        let mut node = self.parse_factor();
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    node = Node::Add(Box::new(node), Box::new(self.parse_factor()));
                }
                Token::Minus => {
                    self.advance();
                    node = Node::Sub(Box::new(node), Box::new(self.parse_factor()));
                }
                _ => break,
            }
        }
        node
    }

    fn parse_factor(&mut self) -> Node {
        let mut node = self.parse_primary();
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    node = Node::Mul(Box::new(node), Box::new(self.parse_primary()));
                }
                Token::Slash => {
                    self.advance();
                    node = Node::Div(Box::new(node), Box::new(self.parse_primary()));
                }
                Token::Shl => {
                    self.advance();
                    node = Node::BitShiftLeft(Box::new(node), Box::new(self.parse_primary()));
                }
                Token::Shr => {
                    self.advance();
                    node = Node::BitShiftRight(Box::new(node), Box::new(self.parse_primary()));
                }
                Token::Amp => {
                    self.advance();
                    node = Node::BitAnd(Box::new(node), Box::new(self.parse_primary()));
                }
                _ => break,
            }
        }
        node
    }

    fn parse_primary(&mut self) -> Node {
        let mut node = match self.peek().clone() {
            Token::Int(v) => {
                self.advance();
                Node::IntLiteral(v)
            }
            Token::Float(v) => {
                self.advance();
                Node::FloatLiteral(v)
            }
            Token::Str(v) => {
                self.advance();
                Node::StringLiteral(v)
            }
            Token::Ident(name) => {
                if name == "true" {
                    self.advance();
                    Node::BoolLiteral(true)
                } else if name == "false" {
                    self.advance();
                    Node::BoolLiteral(false)
                } else {
                    self.advance();
                    if *self.peek() == Token::LParen {
                        self.advance(); // consume '('
                        let mut args = Vec::new();
                        while *self.peek() != Token::RParen {
                            args.push(self.parse_expression());
                            if *self.peek() == Token::Comma {
                                self.advance();
                            }
                        }
                        self.expect(Token::RParen);

                        // Trailing closure block support
                        let mut trailing_block = None;
                        if *self.peek() == Token::LBrace {
                            trailing_block = Some(Box::new(self.parse_block()));
                        }

                        self.construct_node_from_call(&name, args, trailing_block)
                    } else {
                        Node::Identifier(name)
                    }
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression();
                self.expect(Token::RParen);
                expr
            }
            Token::LBracket => {
                self.advance();
                let mut args = Vec::new();
                while *self.peek() != Token::RBracket {
                    args.push(self.parse_expression());
                    if *self.peek() == Token::Comma {
                        self.advance();
                    }
                }
                self.expect(Token::RBracket);
                Node::ArrayCreate(args)
            }
            _ => {
                let hint = format!("Unexpected token in expression: {:?}", self.peek());
                self.diagnostic_panic(&hint)
            }
        };

        // Parse suffixes (array indexing, property access)
        loop {
            if *self.peek() == Token::LBracket {
                self.advance();
                let idx = self.parse_expression();
                self.expect(Token::RBracket);
                node = Node::Index(Box::new(node), Box::new(idx));
            } else if *self.peek() == Token::Dot {
                self.advance();
                if let Token::Ident(prop) = self.advance() {
                    node = Node::PropertyGet(Box::new(node), prop);
                } else {
                    self.diagnostic_panic("Expected property name after dot");
                }
            } else {
                break;
            }
        }
        node
    }

    fn construct_node_from_call(
        &self,
        name: &str,
        mut args: Vec<Node>,
        trailing_block: Option<Box<Node>>,
    ) -> Node {
        // Automatically append trailing block if present
        if let Some(b) = trailing_block {
            args.push(*b);
        }

        match name {
            // AST Map generated directly by Agent
            "Print" => Node::Print(Box::new(args.remove(0))),
            "Time" => Node::Time,
            "InitGraphics" => Node::InitGraphics,
            "InitAudio" => Node::InitAudio,
            "GetLastKeypress" => Node::GetLastKeypress,
            "UIWindow" => Node::UIWindow(
                if let Node::StringLiteral(s) = args.remove(0) {
                    s
                } else {
                    self.diagnostic_panic("UIWindow expects exact String ID arg")
                },
                Box::new(args.remove(0)),
                Box::new(args.remove(0)),
            ),
            "UILabel" => Node::UILabel(Box::new(args.remove(0))),
            "UIButton" => Node::UIButton(Box::new(args.remove(0))),
            "UITextInput" => Node::UITextInput(Box::new(args.remove(0))),
            "UIScrollArea" => Node::UIScrollArea(
                if let Node::StringLiteral(s) = args.remove(0) {
                    s
                } else {
                    self.diagnostic_panic("UIScrollArea expects exact String ID arg")
                },
                Box::new(args.remove(0)),
            ),
            "UIHorizontal" => Node::UIHorizontal(Box::new(args.remove(0))),
            "UIFullscreen" => Node::UIFullscreen(Box::new(args.remove(0))),
            "UIGrid" => Node::UIGrid(
                if let Node::IntLiteral(i) = args.remove(0) {
                    i
                } else {
                    self.diagnostic_panic("UIGrid expects Int args")
                },
                if let Node::StringLiteral(s) = args.remove(0) {
                    s
                } else {
                    self.diagnostic_panic("UIGrid expects String ID")
                },
                Box::new(args.remove(0)),
            ),
            "UISetStyle" => {
                let r = Box::new(args.remove(0));
                let s = Box::new(args.remove(0));
                let a = Box::new(args.remove(0));
                let f = Box::new(args.remove(0));
                let (i, h) = if args.len() >= 2 {
                    (
                        Some(Box::new(args.remove(0))),
                        Some(Box::new(args.remove(0))),
                    )
                } else {
                    (None, None)
                };
                Node::UISetStyle(r, s, a, f, i, h)
            }
            "Concat" => Node::Concat(Box::new(args.remove(0)), Box::new(args.remove(0))),
            "FileRead" => Node::FileRead(Box::new(args.remove(0))),
            "FSRead" => Node::FSRead(Box::new(args.remove(0))),
            "FSWrite" => Node::FSWrite(Box::new(args.remove(0)), Box::new(args.remove(0))),
            _ => Node::Call(name.to_string(), args), // Default to local Call
        }
    }
}
