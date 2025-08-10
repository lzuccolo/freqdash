use csv::Writer;
use std::fs::File;

use crate::backtest::model::StrategyGridRow;
use crate::backtest::sql::build_flat_sql;
use crate::db::AsyncDatabase;
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
pub struct GridQuery {
    pub exchange: String,
    pub currency: String,
    pub pairlist: String,
    pub start_date: String,
    pub months: usize,
}

pub async fn get_grid_summary(params: &GridQuery) -> Result<Vec<StrategyGridRow>, Box<dyn std::error::Error>> {
    let timeranges = generate_timeranges(&params.start_date, params.months);
    let sql = build_flat_sql(params, &timeranges);

    let client = AsyncDatabase::get_client().await;
    let client = client.lock().await;
    let rows = client.query(&sql, &[]).await.unwrap();

    let mut grouped: HashMap<
        (
            String,
            String,
            String,
            String,
            i32,
            bool,
            String,
            String,
            bool,
            String,
            String,
            bool,
        ),
        Vec<_>,
    > = HashMap::new();

    for row in rows {
        let trailing_stop_positive: f64 = row.get("trailing_stop_positive");
        let trailing_stop_positive_offset: f64 = row.get("trailing_stop_positive_offset");

        let tsp_str = trailing_stop_positive.to_string();
        let tspo_str = trailing_stop_positive_offset.to_string();

        let key = (
            row.get("strategy"),
            row.get("timeframe"),
            row.get("minimal_roi"),
            row.get("stoploss"),
            row.get("max_open_trades"),
            row.get("trailing_stop"),
            tsp_str,
            tspo_str,
            row.get("trailing_only_offset_is_reached"),
            row.get("entry_pricing"),
            row.get("exit_pricing"),
            row.get("check_depth_of_market_enable"),
        );

        grouped.entry(key).or_default().push((
            row,
            trailing_stop_positive,
            trailing_stop_positive_offset,
        ));
    }

    let mut result = Vec::new();

    for (_key, items) in grouped.into_iter() {
        let mut monthly = HashMap::new();
        let mut profits = Vec::new();

        let mut profit_total = 0.0;
        let mut total_trades = 0;
        let mut wins = 0;
        let mut winner_holding_avg_s = 0.0;
        let mut max_drawdown = 0.0;
        let mut rejected_signals = 0;

        for (row, _, _) in &items {
            let timerange: String = row.get("timerange");
            let month = &timerange[0..8];
            let profit: f64 = row.get("profit_total");
            monthly.insert(month.to_string(), profit);
            profits.push(profit);

            profit_total += profit;
            total_trades += row.get::<_, i32>("total_trades");
            wins += row.get::<_, i32>("wins");

            let holding: Option<f64> = row.get("winner_holding_avg_s");
            winner_holding_avg_s += holding.unwrap_or(0.0);

            let drawdown: Option<f64> = row.get("max_drawdown");
            max_drawdown += drawdown.unwrap_or(0.0);

            let rejected: Option<i32> = row.get("rejected_signals");
            rejected_signals += rejected.unwrap_or(0);
        }

        let losses = total_trades - wins;
        let win_rate = if total_trades > 0 {
            wins as f64 / total_trades as f64
        } else {
            0.0
        };
        let loss_rate = if total_trades > 0 {
            losses as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_trade_profit = if total_trades > 0 {
            profit_total / total_trades as f64
        } else {
            0.0
        };
        let expectancy = win_rate * avg_trade_profit - loss_rate * avg_trade_profit;

        let total_gain: f64 = profits.iter().filter(|&&x| x > 0.0).sum();
        let total_loss: f64 = profits.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum();
        let profit_factor = if total_loss > 0.0 {
            total_gain / total_loss
        } else {
            total_gain
        };

        let avg_monthly_profit = mean(&profits);
        let std_monthly_profit = stddev(&profits, avg_monthly_profit);
        let max_profit_month = profits.iter().cloned().fold(f64::MIN, f64::max);
        let min_profit_month = profits.iter().cloned().fold(f64::MAX, f64::min);
        let neg_months = profits.iter().filter(|&&x| x < 0.0).count();

        let (
            strategy,
            timeframe,
            minimal_roi,
            stoploss,
            max_open_trades,
            trailing_stop,
            _,
            _,
            trailing_only_offset_is_reached,
            entry_price,
            exit_price,
            check_depth_of_market_enable,
        ) = &_key;

        let (_, trailing_stop_positive, trailing_stop_positive_offset) = items[0].clone();

        result.push(StrategyGridRow {
            strategy: strategy.clone(),
            timeframe: timeframe.clone(),
            minimal_roi: minimal_roi.clone(),
            stoploss: stoploss.clone(),
            max_open_trades: *max_open_trades,
            trailing_stop: *trailing_stop,
            trailing_stop_positive: Some(trailing_stop_positive),
            trailing_stop_positive_offset: Some(trailing_stop_positive_offset),
            trailing_only_offset_is_reached: *trailing_only_offset_is_reached,
            entry_price: entry_price.clone(),
            exit_price: exit_price.clone(),
            check_depth_of_market_enable: *check_depth_of_market_enable,
            total_profit: profit_total,
            total_trades,
            wins,
            win_rate,
            win_time: winner_holding_avg_s / items.len() as f64,
            drawdown_perc: max_drawdown / items.len() as f64,
            rejected_signals: rejected_signals as f64 / items.len() as f64,
            neg_months,
            avg_monthly_profit,
            std_monthly_profit,
            max_profit_month,
            min_profit_month,
            avg_trade_profit,
            losses,
            loss_rate,
            expectancy,
            profit_factor,
            monthly,
        });
    }

    Ok(result)
}

pub async fn get_grid_summary_old(params: &GridQuery) -> Vec<StrategyGridRow> {
    let timeranges = generate_timeranges(&params.start_date, params.months);
    let sql = build_flat_sql(params, &timeranges);

    let client = AsyncDatabase::get_client().await;
    let client = client.lock().await;
    let rows = client.query(&sql, &[]).await.unwrap();

    let mut grouped: HashMap<
        (
            String,
            String,
            String,
            String,
            i32,
            bool,
            String,
            String,
            bool,
            String,
            String,
            bool,
        ),
        Vec<_>,
    > = HashMap::new();

    for row in rows {
        let trailing_stop_positive: f64 = row.get("trailing_stop_positive");
        let trailing_stop_positive_offset: f64 = row.get("trailing_stop_positive_offset");

        let tsp_str = trailing_stop_positive.to_string();
        let tspo_str = trailing_stop_positive_offset.to_string();

        let key = (
            row.get("strategy"),
            row.get("timeframe"),
            row.get("minimal_roi"),
            row.get("stoploss"),
            row.get("max_open_trades"),
            row.get("trailing_stop"),
            tsp_str,
            tspo_str,
            row.get("trailing_only_offset_is_reached"),
            row.get("entry_pricing"),
            row.get("exit_pricing"),
            row.get("check_depth_of_market_enable"),
        );

        grouped.entry(key).or_default().push((
            row,
            trailing_stop_positive,
            trailing_stop_positive_offset,
        ));
    }

    let mut result = Vec::new();

    for (_key, items) in grouped.into_iter() {
        let mut monthly = HashMap::new();
        let mut profits = Vec::new();

        let mut profit_total = 0.0;
        let mut total_trades = 0;
        let mut wins = 0;
        let mut winner_holding_avg_s = 0.0;
        let mut max_drawdown = 0.0;
        let mut rejected_signals = 0;

        for (row, _, _) in &items {
            let timerange: String = row.get("timerange");
            let month = &timerange[0..8];
            let profit: f64 = row.get("profit_total");
            monthly.insert(month.to_string(), profit);
            profits.push(profit);

            profit_total += profit;
            total_trades += row.get::<_, i32>("total_trades");
            wins += row.get::<_, i32>("wins");

            let holding: Option<f64> = row.get("winner_holding_avg_s");
            winner_holding_avg_s += holding.unwrap_or(0.0);

            let drawdown: Option<f64> = row.get("max_drawdown");
            max_drawdown += drawdown.unwrap_or(0.0);

            let rejected: Option<i32> = row.get("rejected_signals");
            rejected_signals += rejected.unwrap_or(0);
        }

        let losses = total_trades - wins;
        let win_rate = if total_trades > 0 {
            wins as f64 / total_trades as f64
        } else {
            0.0
        };
        let loss_rate = if total_trades > 0 {
            losses as f64 / total_trades as f64
        } else {
            0.0
        };
        let avg_trade_profit = if total_trades > 0 {
            profit_total / total_trades as f64
        } else {
            0.0
        };
        let expectancy = win_rate * avg_trade_profit - loss_rate * avg_trade_profit;

        let total_gain: f64 = profits.iter().filter(|&&x| x > 0.0).sum();
        let total_loss: f64 = profits.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum();
        let profit_factor = if total_loss > 0.0 {
            total_gain / total_loss
        } else {
            total_gain
        };

        let avg_monthly_profit = mean(&profits);
        let std_monthly_profit = stddev(&profits, avg_monthly_profit);
        let max_profit_month = profits.iter().cloned().fold(f64::MIN, f64::max);
        let min_profit_month = profits.iter().cloned().fold(f64::MAX, f64::min);
        let neg_months = profits.iter().filter(|&&x| x < 0.0).count();

        let (
            strategy,
            timeframe,
            minimal_roi,
            stoploss,
            max_open_trades,
            trailing_stop,
            _,
            _,
            trailing_only_offset_is_reached,
            entry_price,
            exit_price,
            check_depth_of_market_enable,
        ) = &_key;

        let (_, trailing_stop_positive, trailing_stop_positive_offset) = items[0].clone();

        result.push(StrategyGridRow {
            strategy: strategy.clone(),
            timeframe: timeframe.clone(),
            minimal_roi: minimal_roi.clone(),
            stoploss: stoploss.clone(),
            max_open_trades: *max_open_trades,
            trailing_stop: *trailing_stop,
            trailing_stop_positive: Some(trailing_stop_positive),
            trailing_stop_positive_offset: Some(trailing_stop_positive_offset),
            trailing_only_offset_is_reached: *trailing_only_offset_is_reached,
            entry_price: entry_price.clone(),
            exit_price: exit_price.clone(),
            check_depth_of_market_enable: *check_depth_of_market_enable,
            total_profit: profit_total,
            total_trades,
            wins,
            win_rate,
            win_time: winner_holding_avg_s / items.len() as f64,
            drawdown_perc: max_drawdown / items.len() as f64,
            rejected_signals: rejected_signals as f64 / items.len() as f64,
            neg_months,
            avg_monthly_profit,
            std_monthly_profit,
            max_profit_month,
            min_profit_month,
            avg_trade_profit,
            losses,
            loss_rate,
            expectancy,
            profit_factor,
            monthly,
        });
    }

    result
}

fn generate_timeranges(start_date: &str, months: usize) -> Vec<String> {
    let mut ranges = vec![];
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").unwrap();

    for i in 0..months {
        let from = start
            .with_day(1)
            .unwrap()
            .checked_add_months(chrono::Months::new(i as u32))
            .unwrap();
        let to = if from.month() == 12 {
            NaiveDate::from_ymd_opt(from.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(from.year(), from.month() + 1, 1).unwrap()
        };
        ranges.push(format!("{}-{}", from.format("%Y%m%d"), to.format("%Y%m%d")));
    }

    ranges
}

fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        0.0
    } else {
        data.iter().sum::<f64>() / data.len() as f64
    }
}

fn stddev(data: &[f64], mean: f64) -> f64 {
    let var = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    var.sqrt()
}

pub fn export_summary_to_csv(data: &[StrategyGridRow], filename: &str) {
    let file = File::create(filename).unwrap();
    let mut wtr = Writer::from_writer(file);

    let mut headers = vec![
        "strategy",
        "tf",
        "roi",
        "sl",
        "max_open_trades",
        "ts",
        "tsp",
        "tspo",
        "toor",
        "entry_price",
        "exit_price",
        "depth_mkt",
        "t_profit",
        "t_trades",
        "t_wins",
        "win_rate",
        "win_time",
        "drawdown_perc",
        "rejected_signals",
        "neg_months",
        "avg_monthly_profit",
        "std_monthly_profit",
        "max_profit_month",
        "min_profit_month",
        "avg_trade_profit",
        "losses",
        "loss_rate",
        "expectancy",
        "profit_factor",
    ];

    // Agregamos los nombres de los meses como columnas adicionales
    let mut all_months: Vec<String> = data
        .iter()
        .flat_map(|row| row.monthly.keys().cloned())
        .collect();
    all_months.sort();
    all_months.dedup();
    headers.extend(all_months.iter().map(|s| s.as_str()));

    wtr.write_record(&headers).unwrap();

    for row in data {
        // Convertir campos a String para evitar errores de temporales
        let max_open_trades = row.max_open_trades.to_string();
        let trailing_stop = row.trailing_stop.to_string();
        let tsp = row.trailing_stop_positive.unwrap_or(0.0).to_string();
        let tspo = row.trailing_stop_positive_offset.unwrap_or(0.0).to_string();
        let toor = row.trailing_only_offset_is_reached.to_string();
        let depth_mkt = row.check_depth_of_market_enable.to_string();
        let t_profit = row.total_profit.to_string();
        let t_trades = row.total_trades.to_string();
        let t_wins = row.wins.to_string();
        let win_rate = format!("{:.2}", row.win_rate);
        let win_time = format!("{:.2}", row.win_time);
        let drawdown = format!("{:.2}", row.drawdown_perc);
        let rejected = format!("{:.0}", row.rejected_signals);
        let neg_months = row.neg_months.to_string();
        let avg_monthly = format!("{:.2}", row.avg_monthly_profit);
        let std_monthly = format!("{:.2}", row.std_monthly_profit);
        let max_month = format!("{:.2}", row.max_profit_month);
        let min_month = format!("{:.2}", row.min_profit_month);
        let avg_trade = format!("{:.2}", row.avg_trade_profit);
        let losses = row.losses.to_string();
        let loss_rate = format!("{:.2}", row.loss_rate);
        let expectancy = format!("{:.2}", row.expectancy);
        let profit_factor = format!("{:.2}", row.profit_factor);

        let mut record = vec![
            &row.strategy,
            &row.timeframe,
            &row.minimal_roi,
            &row.stoploss,
            &max_open_trades,
            &trailing_stop,
            &tsp,
            &tspo,
            &toor,
            &row.entry_price,
            &row.exit_price,
            &depth_mkt,
            &t_profit,
            &t_trades,
            &t_wins,
            &win_rate,
            &win_time,
            &drawdown,
            &rejected,
            &neg_months,
            &avg_monthly,
            &std_monthly,
            &max_month,
            &min_month,
            &avg_trade,
            &losses,
            &loss_rate,
            &expectancy,
            &profit_factor,
        ];

        let formatted_months: Vec<String> = all_months
            .iter()
            .map(|month| format!("{:.2}", row.monthly.get(month).cloned().unwrap_or(0.0)))
            .collect();

        for val in &formatted_months {
            record.push(val);
        }

        wtr.write_record(&record).unwrap();
    }

    wtr.flush().unwrap();
    println!("✅ Reporte exportado a '{}'", filename);
}
