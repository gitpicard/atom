use crate::error::*;

#[derive(Copy, Clone, PartialEq, Eq, std::fmt::Debug)]
pub enum TokenType {
    Semicolon,
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
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    NumberLiteral,
    StringLiteral,
    FormattedStringLiteral,
    Identifier,
    TrueLiteral,
    FalseLiteral,
    NullLiteral,
    ThisLiteral,
    SuperLiteral,
    And,
    Or,
    Not,
    Function,
    Class,
    Extends,
    If,
    Else,
    While,
    Do,
    For,
    In,
    Break,
    Continue,
    Return,
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

    pub fn token_data(&self) -> &str {
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
            src_col: 0,
            src: source.chars().peekable(),
        }
    }

    pub fn provide(&mut self, name: &str, source: &'a str) {
        // Reset the scanner to a starting state and provide new code.
        self.src_name = String::from(name);
        self.src_ln = 1;
        self.src_col = 0;
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
            self.src_col = 0;
        }

        c
    }

    fn is_identifier_character(ch: char, is_first: bool) -> bool {
        (is_first && (ch.is_alphabetic() || ch == '_')) || ch.is_alphanumeric() || ch == '_'
    }

    fn str_to_keyword(s: &str) -> Option<TokenType> {
        return match s {
            "true" => Some(TokenType::TrueLiteral),
            "false" => Some(TokenType::FalseLiteral),
            "null" => Some(TokenType::NullLiteral),
            "this" => Some(TokenType::ThisLiteral),
            "super" => Some(TokenType::SuperLiteral),
            "and" => Some(TokenType::And),
            "or" => Some(TokenType::Or),
            "not" => Some(TokenType::Not),
            "function" => Some(TokenType::Function),
            "class" => Some(TokenType::Class),
            "extends" => Some(TokenType::Extends),
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "while" => Some(TokenType::While),
            "do" => Some(TokenType::Do),
            "for" => Some(TokenType::For),
            "in" => Some(TokenType::In),
            "break" => Some(TokenType::Break),
            "continue" => Some(TokenType::Continue),
            "return" => Some(TokenType::Return),
            _ => None,
        };
    }

    fn operator(&self, op: char) -> Option<Token> {
        let operator = match op {
            ';' => Some(TokenType::Semicolon),
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
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            '[' => Some(TokenType::LeftBracket),
            ']' => Some(TokenType::RightBracket),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
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
                        return Some(Err(Error::new(
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
                    return Some(Ok(Token::new(
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
        // Remember where the number started for debug tracking purposes.
        let start_column = self.src_col;

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
            start_column,
            &buffer[..],
        )
    }

    fn consume_string(&mut self, starting: char) -> Result<Token, Error> {
        let mut buffer = String::new();
        let start_column = self.src_col;

        loop {
            // Make sure that we did not run out tokens, this is
            // an error case because the string was not terminated
            // before hitting the end of the source code.
            if let Some(c) = self.pop() {
                if c == starting {
                    // The type of string literal depends on if this is a formatted
                    // string (includes expressions in the string) or just a regular
                    // string.
                    let string_type = if c == '\'' {
                        TokenType::StringLiteral
                    } else {
                        TokenType::FormattedStringLiteral
                    };

                    return Ok(Token::new(
                        string_type,
                        &self.src_name[..],
                        self.src_ln,
                        start_column,
                        &buffer[..],
                    ));
                } else if c == '\\' {
                    // Handle escape characters.
                    buffer.push(match self.pop() {
                        Some('\'') => '\'',
                        Some('"') => '"',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('n') => '\n',
                        Some('\\') => '\\',
                        // Unknown escape characters are not accepted, reject the code.
                        Some(c) => {
                            let msg = format!("unknown escape character {} found", c);
                            return Err(Error::new(
                                &msg[..],
                                &self.src_name[..],
                                self.src_ln,
                                self.src_col,
                            ));
                        }
                        // This happens if there are no more characters after the slash
                        // for an escape character.
                        None => {
                            return Err(Error::new(
                                "expected escape character, found EOF",
                                &self.src_name[..],
                                self.src_ln,
                                self.src_col,
                            ));
                        }
                    });
                } else {
                    // Not the end of the string of an escape characters so just put it
                    // in the buffer.
                    buffer.push(c);
                }
            } else {
                return Err(Error::new(
                    if starting == '"' {
                        "expected \" token"
                    } else {
                        "expected ' token"
                    },
                    &self.src_name[..],
                    self.src_ln,
                    self.src_col,
                ));
            }
        }
    }

    fn consume_identifier(&mut self, starting: char) -> Token {
        // No need to return a result enum here because it is not possible to get
        // a bad identifier in the scanner since we have at least one valid character
        // which makes it valid already. An invalid character is simply not included
        // as part of the identifier.
        let mut buffer = String::from(starting);
        let start_column = self.src_col;

        while Scanner::is_identifier_character(*self.peek().unwrap_or(&'\0'), false) {
            // We can safely unwrap here because we know there is a character here of our
            // is_identifier_character would not have returned true.
            buffer.push(self.pop().unwrap());
        }

        // Check to see if the identifier we found is actually a keyword.
        if let Some(tok_type) = Scanner::str_to_keyword(&buffer[..]) {
            // The function returned a token type which means that it found a
            // keyword from the language.
            return Token::new(
                tok_type,
                &self.src_name[..],
                self.src_ln,
                start_column,
                &buffer[..],
            );
        } else {
            // If no token type was returned, that means the identifier is not
            // a keyword and we can use it as an identifier.
            return Token::new(
                TokenType::Identifier,
                &self.src_name[..],
                self.src_ln,
                start_column,
                &buffer[..],
            );
        }
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
            Some(';') => Some(Ok(self.operator(';').unwrap())),
            Some('+') => Some(Ok(self.operator('+').unwrap())),
            Some('-') => Some(Ok(self.operator('-').unwrap())),
            Some('*') => Some(Ok(self.operator('*').unwrap())),
            Some('/') => Some(Ok(self.operator('/').unwrap())),
            Some('!') => Some(Ok(self.operator('!').unwrap())),
            Some('|') => Some(Ok(self.operator('|').unwrap())),
            Some('&') => Some(Ok(self.operator('&').unwrap())),
            Some('^') => Some(Ok(self.operator('^').unwrap())),
            Some('%') => Some(Ok(self.operator('%').unwrap())),
            Some('~') => Some(Ok(self.operator('~').unwrap())),
            Some('(') => Some(Ok(self.operator('(').unwrap())),
            Some(')') => Some(Ok(self.operator(')').unwrap())),
            Some('[') => Some(Ok(self.operator('[').unwrap())),
            Some(']') => Some(Ok(self.operator(']').unwrap())),
            Some('{') => Some(Ok(self.operator('{').unwrap())),
            Some('}') => Some(Ok(self.operator('}').unwrap())),
            Some('.') if !self.peek().unwrap_or(&'\0').is_ascii_digit() => Some(Ok(self.operator('.').unwrap())),
            // Handle strings which can start with a single quote or a double quote.
            Some('\'') => Some(self.consume_string('\'')),
            Some('"') => Some(self.consume_string('"')),
            Some(c)
                // The pattern guard verifies that we start with a digit or a dot followed
                // by a digit. Just the dot is not enough because it could be the dot operator.
                if c.is_ascii_digit() || (c == '.' && self.peek().unwrap_or(&'\0').is_ascii_digit()) =>
                    // Build a number out of all the digits we can find.
                    Some(Ok(self.consume_number(c))),
            Some(c)
                // Pattern guard makes sure that only identifier characters
                // are let through.
                if Scanner::is_identifier_character(c, true) =>
                    Some(Ok(self.consume_identifier(c))),
            Some(c) => {
                // If we made it this far then we where unable to determine
                // what the token was and we will report the error.
                return Some(Err(Error::new(
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
