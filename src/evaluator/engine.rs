use super::output::Output;
use crate::dsl::ast::Portfolio as PortfolioStatement;
use crate::dsl::ast::{Define, Details, Program, Record, Statement};
use crate::evaluator::output::RecordOutput;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Asset {
    pub symbol: String,
    pub alias: Option<String>,
    pub target_return: Option<f64>,
}

impl Asset {
    pub fn new(symbol: String, alias: Option<String>, target_return: Option<f64>) -> Self {
        Self {
            symbol,
            alias,
            target_return,
        }
    }

    pub fn get_symbol(&self) -> &String {
        &self.symbol
    }

    pub fn get_alias(&self) -> &Option<String> {
        &self.alias
    }

    pub fn get_target_return(&self) -> &Option<f64> {
        &self.target_return
    }
}

#[derive(Clone, Debug)]
pub struct Portfolio {
    pub name: String,
    pub assets: Vec<Asset>,
    pub target_return: f64,
}

pub struct Plan {
    pub name: String,
}

pub struct AnalysisReport {
    // 这里定义有哪些资产, 不包含资产的财务指标
    pub assets: Vec<Asset>,
    // 这里是定义了哪些组合
    pub portfolios: Vec<Portfolio>,

    // 这里就是 DSL 执行完了之后生成的结果, 按照每天进行汇总
    pub daily_snapshot: HashMap<String, Vec<DailySnapshot>>,
}

impl AnalysisReport {
    pub fn new() -> Self {
        Self {
            assets: Vec::new(),
            portfolios: Vec::new(),
            daily_snapshot: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct EngineError {}
impl EngineError {
    pub fn new() -> Self {
        Self {}
    }
}

// record 语句执行后的快照
#[derive(Debug)]
pub struct Snapshot {
    pub symbol: String,
    pub date: String,

    // 执行的 DSL 语句
    pub statement: Record,

    // 执行的结果
    // 总投入
    pub total_purchase: f64,
    // 总转出
    pub total_sale: f64,
    // 期末价值
    pub value: f64,
    // 累积收益, 正负均有可能
    pub profit: f64,
}

impl Snapshot {
    fn from_output(output: &RecordOutput) -> Self {
        Self {
            symbol: output.program.details.get_symbol().to_string(),
            date: output.program.date.clone(),
            statement: output.program.clone(),
            total_purchase: output.total_purchase,
            total_sale: output.total_sale,
            value: output.value,
            profit: output.profit,
        }
    }
}

// 资产的每日快照
// 记录这一天所有的 DSL 脚本及其执行的结果
#[derive(Debug)]
pub struct DailySnapshot {
    pub symbol: String,
    pub date: String,

    pub snapshots: Vec<Snapshot>,
}

impl DailySnapshot {
    fn new(symbol: String, date: String, snapshots: Vec<Snapshot>) -> Self {
        Self {
            symbol,
            date,
            snapshots,
        }
    }
}

struct EngineState {
    portfolios: Vec<Portfolio>,
    plans: Vec<Plan>,
    assets: HashMap<String, Asset>,

    // 每一条 DSL 执行完成都有一个 output,
    record_outputs: Vec<RecordOutput>,

    // 集合output可以生成一个资产每天的最新快照
    // 每天的快照再进行聚合，就能计算出最终的资产价值
    snapshots: HashMap<String, RecordOutput>,
}

#[derive(Debug, Clone)]
struct AssetMetric {
    // 总投入
    total_purchase: f64,
    // 总转出
    total_sale: f64,
    // 期末价值
    value: f64,
    // // 累积收益, 正负均有可能
    // profit: f64,
}

impl AssetMetric {
    fn new_zero() -> Self {
        Self {
            total_purchase: 0.0,
            total_sale: 0.0,
            value: 0.0,
        }
    }

    fn from_record(output: &RecordOutput) -> Self {
        Self {
            total_purchase: output.total_purchase,
            total_sale: output.total_sale,
            value: output.value,
        }
    }

    pub fn get_profit(&self) -> f64 {
        self.value - self.total_purchase + self.total_sale
    }
}

impl EngineState {
    fn new() -> Self {
        Self {
            portfolios: Vec::new(),
            plans: Vec::new(),
            assets: HashMap::new(),
            record_outputs: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    // 如果 asset 存在就更新
    fn upsert_asset(&mut self, args: UpsertAssetArgs) {
        use std::collections::hash_map::Entry;

        match self.assets.entry(args.symbol.to_string()) {
            Entry::Occupied(mut entry) => {
                let asset = entry.get_mut();
                if let Some(name) = args.name {
                    asset.alias = Some(name);
                }

                if let Some(target_return) = args.target_return {
                    asset.target_return = Some(target_return);
                };

                let asset = asset.clone();
                self.update_portfolio_assets(asset);
            }
            Entry::Vacant(entry) => {
                let asset = Asset::new(args.symbol.to_string(), args.name, args.target_return);
                entry.insert(asset);
            }
        }
    }

    // 计算资产的最新的状态
    fn calc_asset(&mut self, record: &Record) -> Result<Output, EngineError> {
        let symbol = record.details.get_symbol().to_string();

        let last = match self.snapshots.get(&symbol) {
            None => AssetMetric::new_zero(),
            Some(shot) => AssetMetric::from_record(shot),
        };

        let details = &record.details;
        let mut new_snapshot = last.clone();

        match details {
            Details::Trade(trade) => {
                let value = trade.signed_amount.value;
                if trade.buy() {
                    new_snapshot.total_purchase = last.total_purchase + value;
                } else {
                    new_snapshot.total_sale = last.total_sale + value;
                }

                // 最新的资产价值
                new_snapshot.value = last.value + trade.signed_amount.to_f64();
            }

            Details::Mark(mark) => {
                // 直接使用 mark 更新资产价值
                new_snapshot.value = mark.value;

                // 使用之前的总投入和转出
                new_snapshot.total_purchase = last.total_purchase;
                new_snapshot.total_sale = last.total_sale;
            }
        }

        let output = RecordOutput::from_record_with_metric(&record, new_snapshot);
        self.record_outputs.push(output.clone());
        self.snapshots.insert(symbol, output.clone());
        Ok(Output::Record(output))
    }

    fn update_portfolio(&mut self, statement: PortfolioStatement) -> Result<(), EngineError> {
        let portfolio = Portfolio {
            name: statement.name.clone(),
            assets: statement
                .assets
                .iter()
                .map(|x| {
                    let symbol = x.to_string();
                    let asset = self
                        .assets
                        .get(&symbol)
                        .cloned()
                        .unwrap_or_else(|| Asset::new(symbol, None, None));

                    asset
                })
                .collect(),
            target_return: statement.target_return.unwrap_or(0.0),
        };

        self.portfolios.push(portfolio);
        Ok(())
    }

    // 更新组合里面的资产名字
    fn update_portfolio_assets(&mut self, asset: Asset) {
        // 遍历每一个组合，如果存在 asset 就更新
        for portfolio in self.portfolios.iter_mut() {
            // 直接使用新的信息覆盖
            for (_i, a) in portfolio.assets.iter_mut().enumerate() {
                if a.get_symbol() == asset.get_symbol() {
                    *a = asset.clone();
                }
            }
        }
    }
}

impl RecordOutput {
    fn from_record_with_metric(record: &Record, metric: AssetMetric) -> Self {
        Self {
            program: record.clone(),
            total_purchase: metric.total_purchase,
            total_sale: metric.total_sale,
            value: metric.value,
            profit: metric.get_profit(),
        }
    }
}

struct UpsertAssetArgs {
    symbol: String,
    name: Option<String>,
    target_return: Option<f64>,
}

struct DefineStatementResult {}

pub struct Engine {
    state: EngineState,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            state: EngineState {
                portfolios: Vec::new(),
                plans: Vec::new(),
                assets: HashMap::new(),
                record_outputs: Vec::new(),
                snapshots: HashMap::new(),
            },
        }
    }

    pub fn evaluate(&mut self, program: Program) -> Result<AnalysisReport, EngineError> {
        let mut record_statements = Vec::new();
        for statement in program.statements.iter() {
            match statement {
                Statement::Record(rec) => record_statements.push(rec),
                Statement::Plan(_) => {}
                Statement::Define(define) => self.evaluate_define(&define)?,
                Statement::Portfolio(statement) => self.evaluate_portfolio(statement)?,
            }
        }

        // 先按照日期排序
        record_statements.sort_by(|a, b| a.date.cmp(&b.date));
        record_statements
            .iter()
            .try_for_each(|rec| self.evaluate_record(rec).map(|_| ()))?;

        let mut result = AnalysisReport::new();
        result.assets = self.state.assets.values().cloned().collect();

        // 按照天聚合每个资产的数据
        for output in self.state.record_outputs.iter() {
            let symbol = output.program.details.get_symbol().to_string();
            let date = output.program.date.clone();

            // 如果不存在就初始化
            let by_symbol = result
                .daily_snapshot
                .entry(symbol.clone())
                .or_insert(Vec::new());

            match by_symbol.iter_mut().find(|x| x.date == date) {
                Some(by_date) => {
                    // 如果存在就直接 push
                    by_date.snapshots.push(Snapshot::from_output(output));
                }
                None => {
                    // 如果不存在就初始化一个然后 push
                    by_symbol.push(DailySnapshot::new(
                        symbol.clone(),
                        date.clone(),
                        vec![Snapshot::from_output(output)],
                    ));
                }
            }
        }

        result.portfolios = self.state.portfolios.clone();
        Ok(result)
    }

    fn evaluate_record(&mut self, record: &Record) -> Result<Output, EngineError> {
        // 先更新资产的基本信息
        let details = &record.details;
        let symbol = details.get_symbol().to_string();
        let args = UpsertAssetArgs {
            symbol,
            name: None,
            target_return: None,
        };
        self.state.upsert_asset(args);

        // 现在计算资产的最新价值等基础财务指标
        let output = self.state.calc_asset(record)?;

        Ok(output)
    }

    fn evaluate_plan(&self, _plan: Plan) -> Result<(), EngineError> {
        Ok(())
    }

    fn evaluate_define(&mut self, define: &Define) -> Result<(), EngineError> {
        self.state.upsert_asset(UpsertAssetArgs {
            symbol: define.symbol.to_string(),
            name: define.alias.clone(),
            target_return: define.target_return.clone(),
        });

        Ok(())
    }

    fn evaluate_portfolio(&mut self, statement: &PortfolioStatement) -> Result<(), EngineError> {
        self.state.update_portfolio(statement.clone())?;

        Ok(())
    }
}
