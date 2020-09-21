use crate::error::*;

#[derive(Copy, Clone)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    Pipe,
    Ampersand,
    Caret,
    Percent,
    Tilde,
    Dot,
    NumberLiteral,
}

pub struct Token {
    tok: TokenType,
    src_name: String,
    src_ln: u32,
    src_col: u32,
    src_data: String,
}

impl Token {
    pub fn new(token: TokenType, name: &str, line: u32, column: u32, data: &str) -> Self {
        Self {
            tok: token,
            src_name: String::from(name),
            src_ln: line,
            src_col: column,
            src_data: String::from(data),
        }
    }

    pub fn token_type(&self) -> TokenType {
        self.tok
    }

    pub fn source_name(&self) -> &str {
        &self.src_name[..]
    }

    pub fn source_line(&self) -> u32 {
        self.src_ln
    }

    pub fn source_column(&self) -> u32 {
        self.src_col
    }

    pub fn data(&self) -> &str {
        &self.src_data[..]
    }
}

pub struct Scanner<'a> {
    src_name: String,
    src_ln: u32,
    src_col: u32,
    src: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(name: &str, source: &'a str) -> Self {
        Self {
            src_name: String::from(name),
            src_ln: 1,
            src_col: 1,
            src: source.chars().peekable(),
        }
    }

    pub fn provide(&mut self, name: &str, source: &'a str) {
        // Reset the scanner to a starting state and provide new code.
        self.src_name = String::from(name);
        self.src_ln = 1;
        self.src_col = 1;
        self.src = source.chars().peekable();
    }

    pub fn source_name(&self) -> &str {
        &self.src_name[..]
    }

    pub fn current_line(&self) -> u32 {
        self.src_ln
    }

    pub fn current_column(&self) -> u32 {
        self.src_col
    }

    fn peek(&mut self) -> Option<&char> {
        self.src.peek()
    }

    fn pop(&mut self) -> Option<char> {
        let c = self.src.next();
        // Keep track of which column we are on for accurate debug
        // and syntax error reporting.
        self.src_col += 1;
        // Make sure to handle new lines.
        if c.is_some() && c.unwrap() == '\n' {
            self.src_ln += 1;
            self.src_col = 1;
        }

        c
    }

    fn operator(&self, op: char) -> Option<Token> {
        let operator = match op {
            '+' => Some(TokenType::Plus),
            '-' => Some(TokenType::Minus),
            '*' => Some(TokenType::Star),
            '/' => Some(TokenType::Star),
            '!' => Some(TokenType::Bang),
            '|' => Some(TokenType::Pipe),
            '&' => Some(TokenType::Ampersand),
            '^' => Some(TokenType::Caret),
            '%' => Some(TokenType::Percent),
            '~' => Some(TokenType::Tilde),
            '.' => Some(TokenType::Dot),
            _ => None,
        };

        // Handle the case that a bad character was passed
        // in that is not an operator.
        if operator.is_none() {
            return None;
        }

        // Create an object filled with data describing the operator
        // that was found.
        Some(Token::new(
            operator.unwrap(),
            &self.src_name[..],
            self.src_ln,
            self.src_col,
            &String::from(op)[..],
        ))
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.pop();
        }
    }

    fn consume_comments(&mut self) -> Option<Result<Token, Error>> {
        // Why loop when looking for a comment to remove? Because there
        // could be several comments in a row before the next token.
        while let Some(&c) = self.peek() {
            if c != '/' {
                break;
            }

            self.pop();
            match self.peek() {
                // Double slash for a single line comment.
                Some('/') => {
                    // Keep consuming until we hit a new line.
                    while let Some(c) = self.pop() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                // Slash and a star for a multi-line comment.
                Some('*') => {
                    let mut hit_end = false;
                    while let Some(c) = self.pop() {
                        // You might think you can optimize away the peek and the pop into
                        // just the pop call but this could result in a **/ not being recognized
                        // as the end of a multi-line comment. Hence, we need to check for the
                        // slash and remove it in two separate steps.
                        if c == '*' && self.peek().unwrap_or(&'\0') == &'/' {
                            self.pop();
                            hit_end = true;
                            break;
                        }
                    }

                    // This checks to see if we hit the end because we found a
                    // end comment token or because we ran out of characters.
                    // Atom does not allow multi-comments to end by reaching the end
                    // of the file. This prevents bugs with mismatched multi-line comments
                    // accidentally commenting out the entire file.
                    if !hit_end {
                        return Some(Result::Err(Error::new(
                            "expected '*/' but found eof",
                            &self.src_name,
                            self.src_ln,
                            self.src_col,
                        )));
                    }
                }
                // We did not see one of the comment start tokens. Which
                // means that we found the single slash which is the slash operator.
                Some(_) => {
                    return Some(Result::Ok(Token::new(
                        TokenType::Slash,
                        &self.src_name,
                        self.src_ln,
                        self.src_col,
                        "/",
                    )))
                }
                // No more source code to look at.
                None => return None,
            }

            // There could be whitespace between here and the start of the
            // next comment.
            self.consume_whitespace();
        }

        None
    }

    fn consume_number(&mut self, starting: char) -> Token {
        let mut dot = if starting == '.' { true } else { false };
        let mut buffer = String::from(starting);

        // Keep consuming digits as long as we can. The scanner is
        // a greedy algorithm.
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                buffer.push(c);
                self.pop();
            } else if c == '.' && !dot {
                buffer.push(c);
                dot = true;
                self.pop();
            } else {
                // This did not match the number so we will finish here.
                break;
            }
        }

        Token::new(
            TokenType::NumberLiteral,
            &self.src_name[..],
            self.src_ln,
            self.src_col,
            &buffer[..],
        )
    }
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();
        // It is possible that while searching for comments
        // to remove, we hit a slash token.
        if let Some(token) = self.consume_comments() {
            return Some(token);
        }

        return match self.pop() {
            // The unwraps here must be safe because we are passing in a character literal
            // and we know that those literals will always match in this case. If the operator
            // function is called from somewhere else, this may not be the case.
            Some('+') => Some(Result::Ok(self.operator('+').unwrap())),
            Some('-') => Some(Result::Ok(self.operator('-').unwrap())),
            Some('*') => Some(Result::Ok(self.operator('*').unwrap())),
            Some('/') => Some(Result::Ok(self.operator('/').unwrap())),
            Some('!') => Some(Result::Ok(self.operator('!').unwrap())),
            Some('|') => Some(Result::Ok(self.operator('|').unwrap())),
            Some('&') => Some(Result::Ok(self.operator('&').unwrap())),
            Some('^') => Some(Result::Ok(self.operator('^').unwrap())),
            Some('%') => Some(Result::Ok(self.operator('%').unwrap())),
            Some('~') => Some(Result::Ok(self.operator('~').unwrap())),
            Some('.') => Some(Result::Ok(self.operator('.').unwrap())),
            Some(c)
                // The pattern guard verifies that we start with a digit or a dot followed
                // by a digit. Just the dot is not enough because it could be the dot operator.
                if c.is_ascii_digit() || (c == '.' && self.peek().unwrap_or(&'\0').is_ascii_digit()) =>
                    // Build a number out of all the digits we can find.
                    Some(Result::Ok(self.consume_number(c))),
            Some(c) => {
                // If we made it this far then we where unable to determine
                // what the token was and we will report the error.
                return Some(Result::Err(Error::new(
                    &format!("unexpected {} token", c)[..],
                    &self.src_name[..],
                    self.src_ln,
                    self.src_col,
                )));
            }
            None => None,
        };
    }
}
