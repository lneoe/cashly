
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Literals
    Date(String),
    Number(f64),
    String(String),
    Identifier(String),
    Symbol(String, String), // namespace:name
    Comment(String),
    
    // Keywords
    Plan,
    Define,
    Portfolio,
    End,
    Trade,
    Mark,
    Schedule,
    Start,
    EndDate,
    Alias,
    Target,
    Return,
    Assets,
    Into,
    Value,
    Note,
    
    // Frequency keywords
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
    
    // Operators
    Plus,
    Minus,
    At,
    
    // Punctuation
    Colon,
    Comma,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    
    // Special
    Newline,
    Eof,
    Invalid(String),
}

impl Token {
    pub fn from_keyword(word: &str) -> Option<Token> {
        match word.to_uppercase().as_str() {
            "PLAN" => Some(Token::Plan),
            "DEFINE" => Some(Token::Define),
            "PORTFOLIO" => Some(Token::Portfolio),
            "END" => Some(Token::End),
            "TRADE" => Some(Token::Trade),
            "MARK" => Some(Token::Mark),
            "SCHEDULE" => Some(Token::Schedule),
            "START" => Some(Token::Start),
            "END_DATE" => Some(Token::EndDate),
            "ALIAS" => Some(Token::Alias),
            "TARGET" => Some(Token::Target),
            "RETURN" => Some(Token::Return),
            "ASSETS" => Some(Token::Assets),
            "INTO" => Some(Token::Into),
            "VALUE" => Some(Token::Value),
            "NOTE" => Some(Token::Note),
            "DAILY" => Some(Token::Daily),
            "WEEKLY" => Some(Token::Weekly),
            "MONTHLY" => Some(Token::Monthly),
            "QUARTERLY" => Some(Token::Quarterly),
            "YEARLY" => Some(Token::Yearly),
            _ => None,
        }
    }
    
    pub fn is_frequency(&self) -> bool {
        matches!(self, Token::Daily | Token::Weekly | Token::Monthly | Token::Quarterly | Token::Yearly)
    }
}
