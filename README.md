### DSL 设计

## EBNF
```
# 顶层结构

<program>      ::= <statement>*
<statement>    ::= <record> | <plan> | <define> | <portfolio>

# 记录语句

<record>       ::= <date> <action> <details> [<note>]

# 投资计划（简化版）

<plan>         ::= "PLAN" <string> <plan_body> "END"
<plan_body>    ::= <plan_rule>*
<plan_rule>    ::= <schedule> | <start_date> | <end_date>
<schedule>     ::= "SCHEDULE" <frequency> <amount> <unit> "INTO" <symbol>
<start_date>   ::= "START" <date>
<end_date>     ::= "END_DATE" <date>
<frequency>    ::= "DAILY" | "WEEKLY" | "MONTHLY" | "QUARTERLY" | "YEARLY"

# 定义元信息

<define>       ::= "DEFINE" <symbol> <define_body> "END"
<define_body>  ::= [ <alias> ] [ <target_return> ]
<alias>        ::= "ALIAS" <string>
<target_return>::= "TARGET" "RETURN" <number>

# 定义组合

<portfolio>      ::= "PORTFOLIO" <string> <portfolio_body> "END"
<portfolio_body> ::= <symbols> [ <target_return> ]
<symbols>        ::= "ASSETS" <symbol_list>
<symbol_list>    ::= <symbol> { "," <symbol> }

# 基础元素

<date>         ::= <year> "-" <month> "-" <day>
<action>       ::= "TRADE" | "MARK"
<details>      ::= <trade_details> | <mark_details>
<trade_details>::= <symbol> <signed_amount> <unit> ["@" <number>]
<mark_details> ::= <symbol> "VALUE" <number> <unit>

# 通用定义

<amount>       ::= <number>
<signed_amount>::= ("+" | "-")? <number>
<unit>         ::= <identifier>
<symbol>       ::= <identifier> ":" <identifier>
<note>         ::= "NOTE" <string>

# 基础类型

<identifier>   ::= [A-Z][A-Z0-9_]*
<number>       ::= [0-9]+ ("." [0-9]+)?
<string>       ::= '"' [^"]* '"'

# 日期组件

<year>         ::= [0-9]{4}
<month>        ::= [0-9]{2}
<day>          ::= [0-9]{2}
```

## DSL 语法示例

#### 定义两个标的
```
DEFINE ETF:510300
ALIAS "沪深300ETF"
TARGET RETURN 0.09
END

DEFINE ETF:159915
ALIAS "创业板ETF"
TARGET RETURN 0.10
END
```

#### 创建组合
```
PORTFOLIO "ETF 长期投资"
  ASSETS ETF:510300, ETF:159915
  TARGET RETURN 0.09
END
```

#### TRADE 语法

```dsl
# 统一交易语法，资产符号前置
2024-01-01 TRADE ETF:510300 +5000 CNY @ 4.56
  NOTE "新年第一笔定投"
2024-02-01 TRADE ETF:510300 +3000 CNY @ 4.32
2024-03-01 TRADE ETF:510300 -2000 CNY @ 4.65
2024-03-31 MARK ETF:510300 VALUE 7200 CNY
  NOTE "第一季度估值"
```

#### 投资计划示例

```dsl
PLAN "2024年定投计划"
  SCHEDULE MONTHLY 3000 CNY INTO ETF:510300
  SCHEDULE MONTHLY 1000 CNY INTO ETF:159915
  START 2024-01-01
  END_DATE 2024-12-31
END
```

#### 完整投资记录示例

```dsl
# 混合使用新旧语法的完整示例
2024-01-02 TRADE ETF:510300 +5000 CNY @ 4.56
2024-01-05 TRADE ETF:510300 +3000 CNY @ 4.62
2024-01-15 MARK ETF:510300 VALUE 8800 CNY
2024-01-19 BUY 4000 CNY OF ETF:159915 @ 2.45            # 传统语法
2024-02-01 TRADE ETF:510300 +6000 CNY @ 4.32
2024-02-15 TRADE ETF:159915 -1000 CNY @ 2.58            # 转出部分
2024-02-29 MARK ETF:510300 VALUE 22800 CNY
2024-02-29 MARK ETF:159915 VALUE 3600 CNY

PLAN "分散投资计划"
  SCHEDULE MONTHLY 2500 CNY INTO ETF:510300
  SCHEDULE MONTHLY 1500 CNY INTO ETF:159915
  START 2024-03-01
END
```
