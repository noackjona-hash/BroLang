#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Set,
    To,
    Print,
    If,
    While,
    End,
    Input,
    Len,
    Sleep,
    Random,
    Ident(String),
    IntLit(i64),
    StrLit(String),
    Plus,
    Minus,
    Star,
    Slash,
    Eq,     // =
    EqEq,   // ==
    NotEq,  // !=
    Lt,     // <
    LtEq,   // <=
    Gt,     // >
    GtEq,   // >=
    LParen, // (
    RParen, // )
    Newline,
}

impl Token {
    pub fn to_string_representation(&self) -> String {
        match self {
            Token::Set => "set/setze".to_string(),
            Token::To => "to/auf".to_string(),
            Token::Print => "print/show/zeige".to_string(),
            Token::If => "if/wenn".to_string(),
            Token::While => "while/solange".to_string(),
            Token::End => "end/ende".to_string(),
            Token::Input => "input/lese".to_string(),
            Token::Len => "len/laenge".to_string(),
            Token::Sleep => "sleep/warte".to_string(),
            Token::Random => "random/zufall".to_string(),
            Token::Ident(s) => format!("identifier '{}'", s),
            Token::IntLit(n) => format!("integer literal '{}'", n),
            Token::StrLit(s) => format!("string literal \"{}\"", s),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Star => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Eq => "=".to_string(),
            Token::EqEq => "==".to_string(),
            Token::NotEq => "!=".to_string(),
            Token::Lt => "<".to_string(),
            Token::LtEq => "<=".to_string(),
            Token::Gt => ">".to_string(),
            Token::GtEq => ">=".to_string(),
            Token::LParen => "(".to_string(),
            Token::RParen => ")".to_string(),
            Token::Newline => "newline".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetaToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.chars.len() {
            return None;
        }
        let ch = self.chars[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    pub fn tokenize(&mut self) -> Result<Vec<MetaToken>, LexError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            // Comments starting with #
            if ch == '#' {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // Newlines
            if ch == '\n' {
                let start_line = self.line;
                let start_col = self.column;
                self.advance();
                // Merge multiple consecutive newlines (or carriage returns + newlines) to avoid cluttering AST parser
                if tokens.last().map(|t: &MetaToken| &t.token) != Some(&Token::Newline) {
                    tokens.push(MetaToken {
                        token: Token::Newline,
                        line: start_line,
                        column: start_col,
                        length: 1,
                    });
                }
                continue;
            }

            if ch == '\r' {
                self.advance();
                continue;
            }

            // Whitespace
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            let start_line = self.line;
            let start_col = self.column;

            // Numbers
            if ch.is_ascii_digit() {
                let mut num_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        num_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                let len = num_str.len();
                let val: i64 = num_str.parse().map_err(|_| LexError {
                    message: format!("Failed to parse integer '{}'", num_str),
                    line: start_line,
                    column: start_col,
                })?;
                tokens.push(MetaToken {
                    token: Token::IntLit(val),
                    line: start_line,
                    column: start_col,
                    length: len,
                });
                continue;
            }

            // Identifiers / Keywords
            if ch.is_alphabetic() || ch == '_' {
                let mut ident_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                let len = ident_str.len();
                let token = match ident_str.as_str() {
                    // English keywords
                    "set" => Token::Set,
                    "to" => Token::To,
                    "print" | "show" => Token::Print,
                    "if" => Token::If,
                    "while" => Token::While,
                    "end" => Token::End,
                    "input" => Token::Input,
                    "len" => Token::Len,
                    "sleep" => Token::Sleep,
                    "random" => Token::Random,

                    // German keywords
                    "setze" => Token::Set,
                    "auf" => Token::To,
                    "zeige" => Token::Print,
                    "wenn" => Token::If,
                    "solange" => Token::While,
                    "ende" => Token::End,
                    "lese" => Token::Input,
                    "laenge" => Token::Len,
                    "warte" => Token::Sleep,
                    "zufall" => Token::Random,

                    // Generic Identifier
                    _ => Token::Ident(ident_str),
                };
                tokens.push(MetaToken {
                    token,
                    line: start_line,
                    column: start_col,
                    length: len,
                });
                continue;
            }

            // String literals
            if ch == '"' {
                self.advance(); // consume opening quote
                let mut string_val = String::new();
                let mut closed = false;
                while let Some(c) = self.peek() {
                    if c == '"' {
                        self.advance(); // consume closing quote
                        closed = true;
                        break;
                    } else if c == '\n' {
                        break; // don't allow unescaped multiline strings for simplicity
                    } else {
                        string_val.push(self.advance().unwrap());
                    }
                }
                if !closed {
                    return Err(LexError {
                        message: "Unterminated string literal. Expected matching double quote (\").".to_string(),
                        line: start_line,
                        column: start_col,
                    });
                }
                let total_len = string_val.len() + 2;
                tokens.push(MetaToken {
                    token: Token::StrLit(string_val),
                    line: start_line,
                    column: start_col,
                    length: total_len,
                });
                continue;
            }

            // Operators & Delimiters
            let tok = match ch {
                '+' => { self.advance(); Some((Token::Plus, 1)) }
                '-' => { self.advance(); Some((Token::Minus, 1)) }
                '*' => { self.advance(); Some((Token::Star, 1)) }
                '/' => { self.advance(); Some((Token::Slash, 1)) }
                '(' => { self.advance(); Some((Token::LParen, 1)) }
                ')' => { self.advance(); Some((Token::RParen, 1)) }
                '=' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Some((Token::EqEq, 2))
                    } else {
                        Some((Token::Eq, 1))
                    }
                }
                '!' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Some((Token::NotEq, 2))
                    } else {
                        return Err(LexError {
                            message: "Unexpected character '!'. Did you mean '!='?".to_string(),
                            line: start_line,
                            column: start_col,
                        });
                    }
                }
                '<' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Some((Token::LtEq, 2))
                    } else {
                        Some((Token::Lt, 1))
                    }
                }
                '>' => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Some((Token::GtEq, 2))
                    } else {
                        Some((Token::Gt, 1))
                    }
                }
                _ => None,
            };

            if let Some((token, len)) = tok {
                tokens.push(MetaToken {
                    token,
                    line: start_line,
                    column: start_col,
                    length: len,
                });
            } else {
                return Err(LexError {
                    message: format!("Unexpected character '{}'", ch),
                    line: start_line,
                    column: start_col,
                });
            }
        }

        Ok(tokens)
    }
}
