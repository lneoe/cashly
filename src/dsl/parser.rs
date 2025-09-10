use super::ast::*;
use super::token::Token;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();

        while !self.is_at_end() {
            // 跳过注释
            self.skip_comments_and_whitespace();
            if self.is_at_end() {
                // 跳过注释后有可能直接就到了末尾了
                break;
            }

            match self.parse_statement() {
                Ok(statement) => program.add_statement(statement),
                Err(e) => return Err(e),
            }
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Skip comments
        while matches!(self.peek(), Token::Comment(_)) {
            self.advance();
        }

        match self.peek() {
            Token::Date(_) => self.parse_record().map(Statement::Record),
            Token::Plan => self.parse_plan().map(Statement::Plan),
            Token::Define => self.parse_define().map(Statement::Define),
            Token::Portfolio => self.parse_portfolio().map(Statement::Portfolio),
            Token::Eof => Err(ParseError {
                message: "Unexpected end of input".to_string(),
                position: self.current,
            }),
            _ => Err(ParseError {
                message: format!("Expected statement, found {:?}", self.peek()),
                position: self.current,
            }),
        }
    }

    fn parse_record(&mut self) -> Result<Record, ParseError> {
        let date = match self.advance() {
            Token::Date(d) => d,
            _ => {
                return Err(ParseError {
                    message: "Expected date".to_string(),
                    position: self.current - 1,
                });
            }
        };

        let action = self.parse_action()?;
        let details = self.parse_details(&action)?;
        let note = self.parse_optional_note()?;

        Ok(Record {
            date,
            action,
            details,
            note,
        })
    }

    fn parse_action(&mut self) -> Result<Action, ParseError> {
        match self.advance() {
            Token::Trade => Ok(Action::Trade),
            Token::Mark => Ok(Action::Mark),
            token => Err(ParseError {
                message: format!("Expected TRADE or MARK, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    fn parse_details(&mut self, action: &Action) -> Result<Details, ParseError> {
        match action {
            Action::Trade => self.parse_trade_details().map(Details::Trade),
            Action::Mark => self.parse_mark_details().map(Details::Mark),
        }
    }

    fn parse_trade_details(&mut self) -> Result<TradeDetails, ParseError> {
        let symbol = self.parse_symbol()?;
        let signed_amount = self.parse_signed_amount()?;
        let unit = self.parse_identifier()?;

        let price = if self.check(&Token::At) {
            self.advance(); // consume '@'
            Some(self.parse_number()?)
        } else {
            None
        };

        Ok(TradeDetails {
            symbol,
            signed_amount,
            unit,
            price,
        })
    }

    fn parse_mark_details(&mut self) -> Result<MarkDetails, ParseError> {
        let symbol = self.parse_symbol()?;
        self.consume(&Token::Value, "Expected VALUE")?;
        let value = self.parse_number()?;
        let unit = self.parse_identifier()?;

        Ok(MarkDetails {
            symbol,
            value,
            unit,
        })
    }

    fn parse_plan(&mut self) -> Result<Plan, ParseError> {
        self.consume(&Token::Plan, "Expected PLAN")?;
        let name = self.parse_string()?;
        let rules = self.parse_plan_body()?;
        self.consume(&Token::End, "Expected END")?;

        Ok(Plan { name, rules })
    }

    fn parse_plan_body(&mut self) -> Result<Vec<PlanRule>, ParseError> {
        let mut rules = Vec::new();

        while !self.check(&Token::End) && !self.is_at_end() {
            let rule = self.parse_plan_rule()?;
            rules.push(rule);
        }

        Ok(rules)
    }

    fn parse_plan_rule(&mut self) -> Result<PlanRule, ParseError> {
        match self.peek() {
            Token::Schedule => {
                self.advance(); // consume SCHEDULE
                let schedule = self.parse_schedule()?;
                Ok(PlanRule::Schedule(schedule))
            }
            Token::Start => {
                self.advance(); // consume START
                let date = self.parse_date()?;
                Ok(PlanRule::StartDate(date))
            }
            Token::EndDate => {
                self.advance(); // consume END_DATE
                let date = self.parse_date()?;
                Ok(PlanRule::EndDate(date))
            }
            token => Err(ParseError {
                message: format!("Expected SCHEDULE, START, or END_DATE, found {:?}", token),
                position: self.current,
            }),
        }
    }

    fn parse_schedule(&mut self) -> Result<Schedule, ParseError> {
        let frequency = self.parse_frequency()?;
        let amount = self.parse_number()?;
        let unit = self.parse_identifier()?;
        self.consume(&Token::Into, "Expected INTO")?;
        let target = self.parse_symbol()?;

        Ok(Schedule {
            frequency,
            amount,
            unit,
            target,
        })
    }

    fn parse_frequency(&mut self) -> Result<Frequency, ParseError> {
        match self.advance() {
            Token::Daily => Ok(Frequency::Daily),
            Token::Weekly => Ok(Frequency::Weekly),
            Token::Monthly => Ok(Frequency::Monthly),
            Token::Quarterly => Ok(Frequency::Quarterly),
            Token::Yearly => Ok(Frequency::Yearly),
            token => Err(ParseError {
                message: format!("Expected frequency, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    fn parse_define(&mut self) -> Result<Define, ParseError> {
        self.consume(&Token::Define, "Expected DEFINE")?;
        let symbol = self.parse_symbol()?;
        let (alias, target_return) = self.parse_define_body()?;
        self.consume(&Token::End, "Expected END")?;

        Ok(Define {
            symbol,
            alias,
            target_return,
        })
    }

    fn parse_define_body(&mut self) -> Result<(Option<String>, Option<f64>), ParseError> {
        let mut alias = None;
        let mut target_return = None;

        while !self.check(&Token::End) && !self.is_at_end() {
            match self.peek() {
                Token::Alias => {
                    self.advance(); // consume ALIAS
                    alias = Some(self.parse_string()?);
                }
                Token::Target => {
                    self.advance(); // consume TARGET
                    self.consume(&Token::Return, "Expected RETURN after TARGET")?;
                    target_return = Some(self.parse_number()?);
                }
                _ => break,
            }
        }

        Ok((alias, target_return))
    }

    fn parse_portfolio(&mut self) -> Result<Portfolio, ParseError> {
        self.consume(&Token::Portfolio, "Expected PORTFOLIO")?;
        let name = self.parse_string()?;
        let (assets, target_return) = self.parse_portfolio_body()?;
        self.consume(&Token::End, "Expected END")?;

        Ok(Portfolio {
            name,
            assets,
            target_return,
        })
    }

    fn parse_portfolio_body(&mut self) -> Result<(Vec<Symbol>, Option<f64>), ParseError> {
        let mut assets = Vec::new();
        let mut target_return = None;

        while !self.check(&Token::End) && !self.is_at_end() {
            match self.peek() {
                Token::Assets => {
                    self.advance(); // consume ASSETS
                    assets = self.parse_symbol_list()?;
                }
                Token::Target => {
                    self.advance(); // consume TARGET
                    self.consume(&Token::Return, "Expected RETURN after TARGET")?;
                    target_return = Some(self.parse_number()?);
                }
                _ => break,
            }
        }

        Ok((assets, target_return))
    }

    fn parse_symbol_list(&mut self) -> Result<Vec<Symbol>, ParseError> {
        let mut symbols = Vec::new();

        symbols.push(self.parse_symbol()?);

        while self.check(&Token::Comma) {
            self.advance(); // consume ','
            symbols.push(self.parse_symbol()?);
        }

        Ok(symbols)
    }

    fn parse_optional_note(&mut self) -> Result<Option<String>, ParseError> {
        if self.check(&Token::Note) {
            self.advance(); // consume NOTE
            Ok(Some(self.parse_string()?))
        } else {
            Ok(None)
        }
    }

    // Helper parsing methods

    fn parse_symbol(&mut self) -> Result<Symbol, ParseError> {
        match self.advance() {
            Token::Symbol(namespace, name) => Ok(Symbol::new(namespace, name)),
            token => Err(ParseError {
                message: format!("Expected symbol, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    fn parse_signed_amount(&mut self) -> Result<SignedAmount, ParseError> {
        let sign = match self.peek() {
            Token::Plus => {
                self.advance();
                Sign::Positive
            }
            Token::Minus => {
                self.advance();
                Sign::Negative
            }
            _ => Sign::Positive, // Default to positive if no sign
        };

        let value = self.parse_number()?;
        Ok(SignedAmount::new(sign, value))
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        match self.advance() {
            Token::Number(n) => Ok(n),
            token => Err(ParseError {
                message: format!("Expected number, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::String(s) => Ok(s),
            token => Err(ParseError {
                message: format!("Expected string, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(s) => Ok(s),
            token => Err(ParseError {
                message: format!("Expected identifier, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    fn parse_date(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Date(d) => Ok(d),
            token => Err(ParseError {
                message: format!("Expected date, found {:?}", token),
                position: self.current - 1,
            }),
        }
    }

    // Utility methods

    fn skip_comments_and_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                Token::Comment(_) | Token::Newline => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof) || self.current >= self.tokens.len()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::Eof)
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }

        match (self.peek(), token_type) {
            (Token::Plan, Token::Plan) => true,
            (Token::Define, Token::Define) => true,
            (Token::Portfolio, Token::Portfolio) => true,
            (Token::End, Token::End) => true,
            (Token::Trade, Token::Trade) => true,
            (Token::Mark, Token::Mark) => true,
            (Token::Schedule, Token::Schedule) => true,
            (Token::Start, Token::Start) => true,
            (Token::EndDate, Token::EndDate) => true,
            (Token::Alias, Token::Alias) => true,
            (Token::Target, Token::Target) => true,
            (Token::Return, Token::Return) => true,
            (Token::Assets, Token::Assets) => true,
            (Token::Into, Token::Into) => true,
            (Token::Value, Token::Value) => true,
            (Token::Note, Token::Note) => true,
            (Token::At, Token::At) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Plus, Token::Plus) => true,
            (Token::Minus, Token::Minus) => true,
            _ => false,
        }
    }

    fn consume(&mut self, expected: &Token, error_message: &str) -> Result<(), ParseError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("{}, found {:?}", error_message, self.peek()),
                position: self.current,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::lexer::Lexer;

    fn parse_input(input: &str) -> Result<Program, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_simple_trade() {
        let input = r#"2024-01-01 TRADE ETF:510300 +5000 CNY @ 4.56"#;
        let program = parse_input(input).unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Record(record) = &program.statements[0] {
            assert_eq!(record.date, "2024-01-01");
            assert_eq!(record.action, Action::Trade);

            if let Details::Trade(details) = &record.details {
                assert_eq!(details.symbol.namespace, "ETF");
                assert_eq!(details.symbol.name, "510300");
                assert_eq!(details.signed_amount.to_f64(), 5000.0);
                assert_eq!(details.unit, "CNY");
                assert_eq!(details.price, Some(4.56));
            } else {
                panic!("Expected trade details");
            }
        } else {
            panic!("Expected record statement");
        }
    }

    #[test]
    fn test_parse_mark_statement() {
        let input = r#"2024-03-31 MARK ETF:510300 VALUE 7200 CNY"#;
        let program = parse_input(input).unwrap();

        assert_eq!(program.statements.len(), 1);

        if let Statement::Record(record) = &program.statements[0] {
            assert_eq!(record.date, "2024-03-31");
            assert_eq!(record.action, Action::Mark);

            if let Details::Mark(details) = &record.details {
                assert_eq!(details.symbol.namespace, "ETF");
                assert_eq!(details.symbol.name, "510300");
                assert_eq!(details.value, 7200.0);
                assert_eq!(details.unit, "CNY");
            } else {
                panic!("Expected mark details");
            }
        } else {
            panic!("Expected record statement");
        }
    }

    #[test]
    fn test_parse_plan() {
        let input = r#"
        PLAN "Investment Plan 2024"
            SCHEDULE MONTHLY 3000 CNY INTO ETF:510300
            SCHEDULE MONTHLY 1000 CNY INTO ETF:159915
            START 2024-01-01
            END_DATE 2024-12-31
        END
        "#;

        let program = parse_input(input).unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::Plan(plan) = &program.statements[0] {
            assert_eq!(plan.name, "Investment Plan 2024");
            assert_eq!(plan.rules.len(), 4);

            if let PlanRule::Schedule(schedule) = &plan.rules[0] {
                assert_eq!(schedule.frequency, Frequency::Monthly);
                assert_eq!(schedule.amount, 3000.0);
                assert_eq!(schedule.unit, "CNY");
                assert_eq!(schedule.target.namespace, "ETF");
                assert_eq!(schedule.target.name, "510300");
            } else {
                panic!("Expected schedule rule");
            }
        } else {
            panic!("Expected plan statement");
        }
    }

    #[test]
    fn test_parse_define() {
        let input = r#"
        DEFINE ETF:510300
            ALIAS "CSI 300 ETF"
            TARGET RETURN 0.09
        END
        "#;

        let program = parse_input(input).unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::Define(define) = &program.statements[0] {
            assert_eq!(define.symbol.namespace, "ETF");
            assert_eq!(define.symbol.name, "510300");
            assert_eq!(define.alias, Some("CSI 300 ETF".to_string()));
            assert_eq!(define.target_return, Some(0.09));
        } else {
            panic!("Expected define statement");
        }
    }

    #[test]
    fn test_parse_portfolio() {
        let input = r#"
        PORTFOLIO "Long Term ETF Investment"
            ASSETS ETF:510300, ETF:159915
            TARGET RETURN 0.09
        END
        "#;

        let program = parse_input(input).unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::Portfolio(portfolio) = &program.statements[0] {
            assert_eq!(portfolio.name, "Long Term ETF Investment");
            assert_eq!(portfolio.assets.len(), 2);
            assert_eq!(portfolio.assets[0].namespace, "ETF");
            assert_eq!(portfolio.assets[0].name, "510300");
            assert_eq!(portfolio.assets[1].namespace, "ETF");
            assert_eq!(portfolio.assets[1].name, "159915");
            assert_eq!(portfolio.target_return, Some(0.09));
        } else {
            panic!("Expected portfolio statement");
        }
    }

    #[test]
    fn test_parse_with_note() {
        let input = r#"2024-01-01 TRADE ETF:510300 +5000 CNY @ 4.56
        NOTE "First investment of the year""#;

        let program = parse_input(input).unwrap();
        assert_eq!(program.statements.len(), 1);

        if let Statement::Record(record) = &program.statements[0] {
            assert_eq!(
                record.note,
                Some("First investment of the year".to_string())
            );
        } else {
            panic!("Expected record statement");
        }
    }

    #[test]
    fn test_parse_negative_amount() {
        let input = r#"2024-03-01 TRADE ETF:510300 -2000 CNY @ 4.65"#;
        let program = parse_input(input).unwrap();

        if let Statement::Record(record) = &program.statements[0] {
            if let Details::Trade(details) = &record.details {
                assert_eq!(details.signed_amount.to_f64(), -2000.0);
            } else {
                panic!("Expected trade details");
            }
        } else {
            panic!("Expected record statement");
        }
    }
}
