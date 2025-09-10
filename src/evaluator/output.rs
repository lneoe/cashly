use crate::dsl::ast::{Record, Symbol};

pub struct AssetMetric {
    // 总投入
    pub total_purchase: f64,
    // 总转出
    pub total_sale: f64,
    // 期末价值
    pub value: f64,
    // 累积收益, 正负均有可能
    pub profit: f64,
}

#[derive(Debug, Clone)]
pub struct RecordOutput {
    pub program: Record,

    // 总投入
    pub total_purchase: f64,
    // 总转出
    pub total_sale: f64,
    // 期末价值
    pub value: f64,
    // 累积收益, 正负均有可能
    pub profit: f64,
}

impl RecordOutput {
    pub fn new(program: Record) -> Self {
        Self {
            program,
            total_purchase: 0.0,
            total_sale: 0.0,
            value: 0.0,
            profit: 0.0,
        }
    }
}

pub struct DefineOutput {}

struct PlanSnapshot {}

// 每个资产的每日快照
struct AssetSnapshot {
    date: String,
    symbol: Symbol,
}

pub enum Output {
    Record(RecordOutput),
    Define(DefineOutput),
}
