use crate::lexer::{MetaToken, Token};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

impl Op {
    pub fn to_string_representation(&self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Eq => "==",
            Op::NotEq => "!=",
            Op::Lt => "<",
            Op::LtEq => "<=",
            Op::Gt => ">",
            Op::GtEq => ">=",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Str(String),
    Var(String),
    Binary {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign {
        name: String,
        value: Expr,
        name_line: usize,
        name_col: usize,
    },
    Print(Expr),
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
}

pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub suggestion: String,
}

pub struct Parser<'a> {
    tokens: &'a [MetaToken],
    pos: usize,
    source_lines: &'a [String],
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [MetaToken], source_lines: &'a [String]) -> Self {
        Self {
            tokens,
            pos: 0,
            source_lines,
        }
    }

    fn peek(&self) -> Option<&MetaToken> {
        if self.pos < self.tokens.len() {
            Some(&self.tokens[self.pos])
        } else {
            None
        }
    }

    fn peek_token(&self) -> Option<&Token> {
        self.peek().map(|mt| &mt.token)
    }

    fn advance(&mut self) -> Option<&MetaToken> {
        if self.pos < self.tokens.len() {
            let res = &self.tokens[self.pos];
            self.pos += 1;
            Some(res)
        } else {
            None
        }
    }

    fn expect_statement_end(&mut self, context: &str) -> Result<(), ParseError> {
        if let Some(mt) = self.peek() {
            match &mt.token {
                Token::Newline => {
                    self.advance();
                    Ok(())
                }
                Token::End => {
                    // Do not consume 'end' / 'ende' as it's the block closer
                    Ok(())
                }
                _ => {
                    Err(ParseError {
                        message: format!("Expected newline after {}, but found {}.", context, mt.token.to_string_representation()),
                        line: mt.line,
                        column: mt.column,
                        length: mt.length,
                        suggestion: format!("Put a newline after this statement or separate your instructions onto new lines."),
                    })
                }
            }
        } else {
            Ok(()) // EOF acts as statement end
        }
    }

    fn parse_block(&mut self, block_name: &str) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while let Some(mt) = self.peek() {
            if mt.token == Token::End {
                return Ok(stmts);
            }
            if mt.token == Token::Newline {
                self.advance();
                continue;
            }
            stmts.push(self.parse_statement()?);
        }

        // Hit EOF without finding end block closure
        let last_line = self.source_lines.len();
        let last_col = self.source_lines.last().map(|l| l.len() + 1).unwrap_or(1);
        Err(ParseError {
            message: format!("Unclosed '{}' block. Reached the end of the file without finding 'end' or 'ende'.", block_name),
            line: last_line,
            column: last_col,
            length: 1,
            suggestion: format!("Add 'end' (or 'ende' in German) to close the '{}' block.", block_name),
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let mt = self.peek().cloned().ok_or_else(|| {
            let last_line = self.source_lines.len();
            let last_col = self.source_lines.last().map(|l| l.len() + 1).unwrap_or(1);
            ParseError {
                message: "Unexpected end of file while parsing statement.".to_string(),
                line: last_line,
                column: last_col,
                length: 1,
                suggestion: "Add a statement to complete the program.".to_string(),
            }
        })?;

        match &mt.token {
            Token::Set => {
                self.advance(); // consume 'set'
                let name_mt = self.peek().cloned().ok_or_else(|| ParseError {
                    message: "Expected variable name after 'set'/'setze'.".to_string(),
                    line: mt.line,
                    column: mt.column + mt.length + 1,
                    length: 1,
                    suggestion: "Specify a variable name, e.g., 'set counter to 5'.".to_string(),
                })?;

                let name = match &name_mt.token {
                    Token::Ident(s) => {
                        let s = s.clone();
                        self.advance();
                        s
                    }
                    _ => {
                        return Err(ParseError {
                            message: format!("Expected variable name after 'set'/'setze', but found {}.", name_mt.token.to_string_representation()),
                            line: name_mt.line,
                            column: name_mt.column,
                            length: name_mt.length,
                            suggestion: "Variable names must start with a letter and contain only letters, numbers, or underscores.".to_string(),
                        });
                    }
                };

                let to_mt = self.peek().cloned().ok_or_else(|| ParseError {
                    message: format!("Expected 'to' or 'auf' after variable name '{}'.", name),
                    line: name_mt.line,
                    column: name_mt.column + name_mt.length + 1,
                    length: 1,
                    suggestion: format!("Use 'to' (or 'auf' in German) to assign a value, e.g., 'set {} to 10'.", name),
                })?;

                match &to_mt.token {
                    Token::To => {
                        self.advance();
                    }
                    Token::Eq => {
                        return Err(ParseError {
                            message: "BroLang does not use '=' for variable assignment.".to_string(),
                            line: to_mt.line,
                            column: to_mt.column,
                            length: to_mt.length,
                            suggestion: format!("Use 'to' (or 'auf' in German) instead of '=', e.g.: 'set {} to [value]' or 'setze {} auf [value]'.", name, name),
                        });
                    }
                    _ => {
                        return Err(ParseError {
                            message: format!("Expected 'to' or 'auf' after variable name '{}', but found {}.", name, to_mt.token.to_string_representation()),
                            line: to_mt.line,
                            column: to_mt.column,
                            length: to_mt.length,
                            suggestion: format!("Change this to 'to' (or 'auf' in German) to assign a value, e.g.: 'set {} to ...' / 'setze {} auf ...'", name, name),
                        });
                    }
                }

                let value = self.parse_expr()?;
                self.expect_statement_end(&format!("variable assignment '{}'", name))?;

                Ok(Stmt::Assign {
                    name,
                    value,
                    name_line: name_mt.line,
                    name_col: name_mt.column,
                })
            }
            Token::Print => {
                self.advance(); // consume 'print' / 'show' / 'zeige'
                let has_paren = if self.peek_token() == Some(&Token::LParen) {
                    self.advance();
                    true
                } else {
                    false
                };

                let expr = self.parse_expr()?;

                if has_paren {
                    let rp_mt = self.peek().cloned().ok_or_else(|| ParseError {
                        message: "Expected closing parenthesis ')' after print expression.".to_string(),
                        line: mt.line,
                        column: mt.column + mt.length + 1,
                        length: 1,
                        suggestion: "Add a closing parenthesis ')' at the end of print, e.g., 'print(5)'.".to_string(),
                    })?;
                    if rp_mt.token == Token::RParen {
                        self.advance();
                    } else {
                        return Err(ParseError {
                            message: format!("Expected closing parenthesis ')' after print expression, but found {}.", rp_mt.token.to_string_representation()),
                            line: rp_mt.line,
                            column: rp_mt.column,
                            length: rp_mt.length,
                            suggestion: "Replace this with ')' or add a matching closing parenthesis.".to_string(),
                        });
                    }
                }

                self.expect_statement_end("print statement")?;
                Ok(Stmt::Print(expr))
            }
            Token::If => {
                self.advance(); // consume 'if' / 'wenn'
                let cond = self.parse_expr()?;
                self.expect_statement_end("if condition")?;
                let then_branch = self.parse_block("if")?;

                // Check and consume 'end' / 'ende'
                let end_mt = self.peek().cloned().ok_or_else(|| ParseError {
                    message: "Expected 'end' or 'ende' to close 'if' block.".to_string(),
                    line: mt.line,
                    column: mt.column,
                    length: mt.length,
                    suggestion: "Add 'end' (or 'ende') to terminate the conditional block.".to_string(),
                })?;
                if end_mt.token == Token::End {
                    self.advance();
                } else {
                    return Err(ParseError {
                        message: format!("Expected 'end' or 'ende' to close 'if' block, but found {}.", end_mt.token.to_string_representation()),
                        line: end_mt.line,
                        column: end_mt.column,
                        length: end_mt.length,
                        suggestion: "Change this to 'end' / 'ende' to terminate the conditional block.".to_string(),
                    });
                }
                self.expect_statement_end("end of if block")?;
                Ok(Stmt::If { cond, then_branch })
            }
            Token::While => {
                self.advance(); // consume 'while' / 'solange'
                let cond = self.parse_expr()?;
                self.expect_statement_end("while condition")?;
                let body = self.parse_block("while")?;

                // Check and consume 'end' / 'ende'
                let end_mt = self.peek().cloned().ok_or_else(|| ParseError {
                    message: "Expected 'end' or 'ende' to close 'while' block.".to_string(),
                    line: mt.line,
                    column: mt.column,
                    length: mt.length,
                    suggestion: "Add 'end' (or 'ende') to terminate the loop block.".to_string(),
                })?;
                if end_mt.token == Token::End {
                    self.advance();
                } else {
                    return Err(ParseError {
                        message: format!("Expected 'end' or 'ende' to close 'while' block, but found {}.", end_mt.token.to_string_representation()),
                        line: end_mt.line,
                        column: end_mt.column,
                        length: end_mt.length,
                        suggestion: "Change this to 'end' / 'ende' to terminate the loop block.".to_string(),
                    });
                }
                self.expect_statement_end("end of while block")?;
                Ok(Stmt::While { cond, body })
            }
            _ => {
                Err(ParseError {
                    message: format!("Unexpected token '{}' at start of statement.", mt.token.to_string_representation()),
                    line: mt.line,
                    column: mt.column,
                    length: mt.length,
                    suggestion: "Expected an instruction like variable assignment (set/setze), output (print/show/zeige), a loop (while/solange), or a conditional (if/wenn).".to_string(),
                })
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_add_sub()?;
        while let Some(mt) = self.peek() {
            let op = match &mt.token {
                Token::EqEq => Op::Eq,
                Token::NotEq => Op::NotEq,
                Token::Lt => Op::Lt,
                Token::LtEq => Op::LtEq,
                Token::Gt => Op::Gt,
                Token::GtEq => Op::GtEq,
                _ => break,
            };
            self.advance(); // consume op
            let right = self.parse_add_sub()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_div()?;
        while let Some(mt) = self.peek() {
            let op = match &mt.token {
                Token::Plus => Op::Add,
                Token::Minus => Op::Sub,
                _ => break,
            };
            self.advance(); // consume op
            let right = self.parse_mul_div()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;
        while let Some(mt) = self.peek() {
            let op = match &mt.token {
                Token::Star => Op::Mul,
                Token::Slash => Op::Div,
                _ => break,
            };
            self.advance(); // consume op
            let right = self.parse_primary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let mt = self.peek().cloned().ok_or_else(|| {
            let last_line = self.source_lines.len();
            let last_col = self.source_lines.last().map(|l| l.len() + 1).unwrap_or(1);
            ParseError {
                message: "Unexpected end of file while parsing expression.".to_string(),
                line: last_line,
                column: last_col,
                length: 1,
                suggestion: "Complete the expression with a number, variable, or string.".to_string(),
            }
        })?;

        match &mt.token {
            Token::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Int(val))
            }
            Token::StrLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::Str(val))
            }
            Token::Ident(name) => {
                let val = name.clone();
                self.advance();
                Ok(Expr::Var(val))
            }
            Token::LParen => {
                self.advance(); // consume '('
                let expr = self.parse_expr()?;
                let rp_mt = self.peek().cloned().ok_or_else(|| ParseError {
                    message: "Expected closing parenthesis ')'.".to_string(),
                    line: mt.line,
                    column: mt.column,
                    length: 1,
                    suggestion: "Add a closing parenthesis ')' to complete the expression.".to_string(),
                })?;
                if rp_mt.token == Token::RParen {
                    self.advance();
                    Ok(expr)
                } else {
                    Err(ParseError {
                        message: format!("Expected closing parenthesis ')', but found {}.", rp_mt.token.to_string_representation()),
                        line: rp_mt.line,
                        column: rp_mt.column,
                        length: rp_mt.length,
                        suggestion: "Replace this token with a closing parenthesis ')' or correct the syntax.".to_string(),
                    })
                }
            }
            _ => {
                Err(ParseError {
                    message: format!("Unexpected token '{}' where expression was expected.", mt.token.to_string_representation()),
                    line: mt.line,
                    column: mt.column,
                    length: mt.length,
                    suggestion: "Expected a number, a variable, a string, or a sub-expression in parentheses like '(x + 5)'.".to_string(),
                })
            }
        }
    }
}

pub fn parse_program(tokens: &[MetaToken], source_lines: &[String]) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens, source_lines);
    let mut statements = Vec::new();
    while let Some(mt) = parser.peek() {
        if mt.token == Token::Newline {
            parser.advance();
            continue;
        }
        statements.push(parser.parse_statement()?);
    }
    Ok(Program { statements })
}

pub fn print_parse_error(err: &ParseError, source_lines: &[String]) {
    eprintln!("\x1b[1;31mError:\x1b[0m {}", err.message);
    eprintln!("At line {}, column {}:", err.line, err.column);
    eprintln!();

    if err.line > 0 && err.line <= source_lines.len() {
        let line_content = &source_lines[err.line - 1];
        eprintln!("  {:3} | {}", err.line, line_content);
        let padding = " ".repeat(err.column - 1);
        let underline = "^".repeat(std::cmp::max(1, err.length));
        eprintln!("      | \x1b[1;31m{}{}\x1b[0m", padding, underline);
    }

    eprintln!();
    eprintln!("\x1b[1;32mSuggestion:\x1b[0m {}", err.suggestion);
}
