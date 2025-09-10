use super::token::Token;

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_line: usize,
    current_column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            current_line: 1,
            current_column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }
            
            let token = self.scan_token()?;
            if !matches!(token, Token::Newline) {
                tokens.push(token);
            }
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_next(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        
        let ch = self.input[self.position];
        self.position += 1;
        
        if ch == '\n' {
            self.current_line += 1;
            self.current_column = 1;
        } else {
            self.current_column += 1;
        }
        
        ch
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => { self.advance(); }
                '\n' => break, // Don't skip newlines as they might be significant
                _ => break,
            }
        }
    }

    fn scan_token(&mut self) -> Result<Token, LexError> {
        let ch = self.advance();
        
        match ch {
            '\n' => Ok(Token::Newline),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '@' => Ok(Token::At),
            ':' => Ok(Token::Colon),
            ',' => Ok(Token::Comma),
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            '"' => self.scan_string(),
            '#' => self.scan_comment(),
            '0'..='9' => {
                self.position -= 1; // Back up
                self.current_column -= 1;
                self.scan_number_or_date()
            }
            'A'..='Z' | 'a'..='z' | '_' => {
                self.position -= 1; // Back up
                self.current_column -= 1;
                self.scan_identifier_or_keyword()
            }
            _ => Err(LexError {
                message: format!("Unexpected character: '{}'", ch),
                position: self.position,
            }),
        }
    }

    fn scan_string(&mut self) -> Result<Token, LexError> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                return Err(LexError {
                    message: "Unterminated string".to_string(),
                    position: self.position,
                });
            }
            
            let ch = self.advance();
            if ch == '\\' {
                // Handle escape sequences
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    ch => {
                        value.push('\\');
                        value.push(ch);
                    }
                }
            } else {
                value.push(ch);
            }
        }
        
        if self.is_at_end() {
            return Err(LexError {
                message: "Unterminated string".to_string(),
                position: self.position,
            });
        }
        
        // Consume closing quote
        self.advance();
        Ok(Token::String(value))
    }

    fn scan_comment(&mut self) -> Result<Token, LexError> {
        let mut comment = String::new();
        
        while !self.is_at_end() && self.peek() != '\n' {
            comment.push(self.advance());
        }
        
        Ok(Token::Comment(comment))
    }

    fn scan_number(&mut self) -> Result<Token, LexError> {
        let mut has_dot = false;
        let start_pos = self.position;
        let is_negative = self.peek() == '-';
        
        if is_negative {
            self.advance();
        }
        
        while !self.is_at_end() {
            match self.peek() {
                '0'..='9' => { self.advance(); }
                '.' if !has_dot => {
                    has_dot = true;
                    self.advance();
                }
                _ => break,
            }
        }
        
        let number_str: String = self.input[start_pos..self.position].iter().collect();
        
        match number_str.parse::<f64>() {
            Ok(num) => Ok(Token::Number(num)),
            Err(_) => Err(LexError {
                message: format!("Invalid number: {}", number_str),
                position: start_pos,
            }),
        }
    }

    fn scan_number_or_date(&mut self) -> Result<Token, LexError> {
        let start_pos = self.position;
        
        // Read first sequence of digits
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }
        
        // Check if this might be a date (YYYY-MM-DD format)
        if self.peek() == '-' && (self.position - start_pos) == 4 {
            // Potentially a date, continue reading
            self.advance(); // consume '-'
            
            // Read month
            let month_start = self.position;
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
            
            if (self.position - month_start) == 2 && self.peek() == '-' {
                self.advance(); // consume second '-'
                
                // Read day
                let day_start = self.position;
                while !self.is_at_end() && self.peek().is_ascii_digit() {
                    self.advance();
                }
                
                if (self.position - day_start) == 2 {
                    // Valid date format
                    let date_str: String = self.input[start_pos..self.position].iter().collect();
                    return Ok(Token::Date(date_str));
                }
            }
        }
        
        // Not a date, check if it's a number
        if self.peek() == '.' {
            self.advance();
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        
        let number_str: String = self.input[start_pos..self.position].iter().collect();
        
        match number_str.parse::<f64>() {
            Ok(num) => Ok(Token::Number(num)),
            Err(_) => Err(LexError {
                message: format!("Invalid number or date: {}", number_str),
                position: start_pos,
            }),
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> Result<Token, LexError> {
        let start_pos = self.position;
        
        // Read identifier characters
        while !self.is_at_end() {
            match self.peek() {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => { self.advance(); }
                _ => break,
            }
        }
        
        let word: String = self.input[start_pos..self.position].iter().collect();
        
        // Check for compound keywords like "END_DATE"
        if word == "END" && self.peek() == '_' {
            self.advance(); // consume '_'

            while !self.is_at_end() {
                match self.peek() {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => { self.advance(); }
                    _ => break,
                }
            }
            
            let compound_word: String = self.input[start_pos..self.position].iter().collect();
            if let Some(token) = Token::from_keyword(&compound_word) {
                return Ok(token);
            }
        }
        
        // Check if it's a symbol (identifier:identifier)
        if self.peek() == ':' {
            self.advance(); // consume ':'
            let namespace = word;
            
            let symbol_start = self.position;
            while !self.is_at_end() {
                match self.peek() {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => { self.advance(); }
                    _ => break,
                }
            }
            
            if self.position > symbol_start {
                let name: String = self.input[symbol_start..self.position].iter().collect();
                return Ok(Token::Symbol(namespace, name));
            } else {
                return Err(LexError {
                    message: "Expected identifier after ':'".to_string(),
                    position: self.position,
                });
            }
        }
        
        // Try to match keyword first
        if let Some(token) = Token::from_keyword(&word) {
            Ok(token)
        } else {
            Ok(Token::Identifier(word))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_tokens() {
        let mut lexer = Lexer::new("+ - @ : , ( ) { }");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Plus,
            Token::Minus,
            Token::At,
            Token::Colon,
            Token::Comma,
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_numbers() {
        let mut lexer = Lexer::new("123 45.67 -89.01");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Number(123.0),
            Token::Number(45.67),
            Token::Minus,
            Token::Number(89.01),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_date() {
        let mut lexer = Lexer::new("2024-01-15");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Date("2024-01-15".to_string()),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new(r#""Hello World" "Test \"Quote\"" "#);
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::String("Hello World".to_string()),
            Token::String("Test \"Quote\"".to_string()),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_keywords() {
        let mut lexer = Lexer::new("PLAN DEFINE END TRADE MARK MONTHLY END_DATE");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Plan,
            Token::Define,
            Token::End,
            Token::Trade,
            Token::Mark,
            Token::Monthly,
            Token::EndDate,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_symbol() {
        let mut lexer = Lexer::new("ETF:510300 STOCK:AAPL");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Symbol("ETF".to_string(), "510300".to_string()),
            Token::Symbol("STOCK".to_string(), "AAPL".to_string()),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_comment() {
        let mut lexer = Lexer::new("# This is a comment\nTRADE");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Comment(" This is a comment".to_string()),
            Token::Trade,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_tokenize_complete_statement() {
        let mut lexer = Lexer::new("2024-01-01 TRADE ETF:510300 +5000 CNY @ 4.56");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Date("2024-01-01".to_string()),
            Token::Trade,
            Token::Symbol("ETF".to_string(), "510300".to_string()),
            Token::Plus,
            Token::Number(5000.0),
            Token::Identifier("CNY".to_string()),
            Token::At,
            Token::Number(4.56),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_standalone_negative_number() {
        let mut lexer = Lexer::new("-42");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens, vec![
            Token::Minus,
            Token::Number(42.0),
            Token::Eof,
        ]);
    }

    #[test]
    fn test_negative_number_in_context() {
        let mut lexer = Lexer::new("2024-01-01 TRADE ETF:510300 -1000 CNY @ 4.56");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens: {:?}", tokens);
        
        assert_eq!(tokens, vec![
            Token::Date("2024-01-01".to_string()),
            Token::Trade,
            Token::Symbol("ETF".to_string(), "510300".to_string()),
            Token::Minus,
            Token::Number(1000.0),
            Token::Identifier("CNY".to_string()),
            Token::At,
            Token::Number(4.56),
            Token::Eof,
        ]);
    }
}
