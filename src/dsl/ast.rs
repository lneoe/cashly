
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Record(Record),
    Plan(Plan),
    Define(Define),
    Portfolio(Portfolio),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Record {
    pub date: String,
    pub action: Action,
    pub details: Details,
    pub note: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Trade,
    Mark,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Details {
    Trade(TradeDetails),
    Mark(MarkDetails),
}

impl Details {
    pub fn get_symbol(&self) -> &Symbol {
        match self {
            Details::Trade(trade) => &trade.symbol,
            Details::Mark(mark) => &mark.symbol,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TradeDetails {
    pub symbol: Symbol,
    pub signed_amount: SignedAmount,
    pub unit: String,
    pub price: Option<f64>,
}

impl TradeDetails {
    pub fn buy(&self) -> bool {
        self.signed_amount.sign == Sign::Positive
    }

    pub fn sell(&self) -> bool {
        self.signed_amount.sign == Sign::Negative
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkDetails {
    pub symbol: Symbol,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Plan {
    pub name: String,
    pub rules: Vec<PlanRule>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PlanRule {
    Schedule(Schedule),
    StartDate(String),
    EndDate(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Schedule {
    pub frequency: Frequency,
    pub amount: f64,
    pub unit: String,
    pub target: Symbol,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Define {
    pub symbol: Symbol,
    pub alias: Option<String>,
    pub target_return: Option<f64>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Portfolio {
    pub name: String,
    pub assets: Vec<Symbol>,
    pub target_return: Option<f64>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    pub namespace: String,
    pub name: String,
}

impl Symbol {
    pub fn new(namespace: String, name: String) -> Self {
        Self { namespace, name }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SignedAmount {
    pub sign: Sign,
    pub value: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Sign {
    Positive,
    Negative,
}

impl SignedAmount {
    pub fn new(sign: Sign, value: f64) -> Self {
        Self { sign, value }
    }

    pub fn positive(value: f64) -> Self {
        Self::new(Sign::Positive, value)
    }

    pub fn negative(value: f64) -> Self {
        Self::new(Sign::Negative, value)
    }

    pub fn to_f64(&self) -> f64 {
        match self.sign {
            Sign::Positive => self.value,
            Sign::Negative => -self.value,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_display() {
        let symbol = Symbol::new("ETF".to_string(), "510300".to_string());
        assert_eq!(format!("{}", symbol), "ETF:510300");
    }

    #[test]
    fn test_signed_amount() {
        let positive = SignedAmount::positive(100.0);
        assert_eq!(positive.to_f64(), 100.0);

        let negative = SignedAmount::negative(50.0);
        assert_eq!(negative.to_f64(), -50.0);
    }

    #[test]
    fn test_program_creation() {
        let mut program = Program::new();

        let record = Record {
            date: "2024-01-01".to_string(),
            action: Action::Trade,
            details: Details::Trade(TradeDetails {
                symbol: Symbol::new("ETF".to_string(), "510300".to_string()),
                signed_amount: SignedAmount::positive(5000.0),
                unit: "CNY".to_string(),
                price: Some(4.56),
            }),
            note: Some("Test trade".to_string()),
        };

        program.add_statement(Statement::Record(record));
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_plan_creation() {
        let plan = Plan {
            name: "Test Plan".to_string(),
            rules: vec![
                PlanRule::Schedule(Schedule {
                    frequency: Frequency::Monthly,
                    amount: 3000.0,
                    unit: "CNY".to_string(),
                    target: Symbol::new("ETF".to_string(), "510300".to_string()),
                }),
                PlanRule::StartDate("2024-01-01".to_string()),
                PlanRule::EndDate("2024-12-31".to_string()),
            ],
        };

        assert_eq!(plan.rules.len(), 3);
        assert_eq!(plan.name, "Test Plan");
    }

    #[test]
    fn test_define_creation() {
        let define = Define {
            symbol: Symbol::new("ETF".to_string(), "510300".to_string()),
            alias: Some("沪深300ETF".to_string()),
            target_return: Some(0.09),
        };

        assert_eq!(define.symbol.namespace, "ETF");
        assert_eq!(define.symbol.name, "510300");
        assert_eq!(define.alias, Some("沪深300ETF".to_string()));
        assert_eq!(define.target_return, Some(0.09));
    }

    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio {
            name: "ETF Portfolio".to_string(),
            assets: vec![
                Symbol::new("ETF".to_string(), "510300".to_string()),
                Symbol::new("ETF".to_string(), "159915".to_string()),
            ],
            target_return: Some(0.09),
        };

        assert_eq!(portfolio.assets.len(), 2);
        assert_eq!(portfolio.name, "ETF Portfolio");
        assert_eq!(portfolio.target_return, Some(0.09));
    }
}
