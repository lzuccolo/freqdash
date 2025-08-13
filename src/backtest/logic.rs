// src/backtest/logic.rs

use chrono::{Datelike, Months, NaiveDate};
use csv::Writer;
use deadpool_postgres::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

use crate::backtest::model::{GridQuery, StrategyGridRow};
use crate::backtest::sql::build_flat_sql;
use crate::db;

// --- SOLUCIÓN AL ERROR DE TIPOS ---
// Creamos un alias para la tupla compleja que usamos como clave.
// Esto ayuda al compilador a inferir los tipos correctamente.
type StrategyKey = (
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
);

// --- FUNCIÓN PRINCIPAL MODIFICADA ---
// Ahora acepta un `&Client` como primer argumento.
pub async fn get_grid_summary(
    client: &Client,
    params: &GridQuery,
) -> Result<Vec<StrategyGridRow>, Box<dyn Error>> {
    let timeranges = generate_timeranges(&params.start_date, params.months);
    let sql = build_flat_sql(params, &timeranges);

    // Ya no usamos el Singleton. El cliente viene como parámetro.
    let rows = client.query(&sql, &[]).await?;

    // El HashMap ahora usa nuestro alias para mayor claridad
    let mut grouped: HashMap<StrategyKey, Vec<_>> = HashMap::new();

    for row in rows {
        let trailing_stop_positive: f64 = row.get("trailing_stop_positive");
        let trailing_stop_positive_offset: f64 = row.get("trailing_stop_positive_offset");
        let tsp_str = trailing_stop_positive.to_string();
        let tspo_str = trailing_stop_positive_offset.to_string();

        let key: StrategyKey = (
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
    let mut total_profit_by_month: HashMap<StrategyKey, Vec<f64>> = HashMap::new();

    // Calculamos los profits mensuales por separado
    for (key, items) in &grouped {
        let mut monthly_profits = Vec::new();
        for (row, _, _) in items {
            monthly_profits.push(row.get("profit_total"));
        }
        total_profit_by_month.insert(key.clone(), monthly_profits);
    }

    for (_key, items) in grouped.into_iter() {
        // --- SOLUCIÓN AL ERROR DE TIPOS ---
        // Le decimos explícitamente al compilador el tipo de la clave.
        let key: &StrategyKey = &_key;
        let (
            strategy,
            timeframe,
            minimal_roi,
            stoploss,
            max_open_trades,
            trailing_stop,
            _,
            _, // tsp_str y tspo_str
            trailing_only_offset_is_reached,
            entry_price,
            exit_price,
            check_depth_of_market_enable,
        ) = key;

        let mut monthly_map = HashMap::new();
        let profits = total_profit_by_month.get(key).unwrap();

        let mut total_trades = 0;
        let mut wins = 0;
        let mut winner_holding_avg_s = 0.0;
        let mut max_drawdown = 0.0;
        let mut rejected_signals = 0;

        for (i, (row, _, _)) in items.iter().enumerate() {
            let timerange: String = row.get("timerange");
            let month = &timerange[0..8];
            monthly_map.insert(month.to_string(), profits[i]);

            total_trades += row.get::<_, i32>("total_trades");
            wins += row.get::<_, i32>("wins");
            winner_holding_avg_s += row
                .get::<_, Option<f64>>("winner_holding_avg_s")
                .unwrap_or(0.0);
            max_drawdown += row.get::<_, Option<f64>>("max_drawdown").unwrap_or(0.0);
            rejected_signals += row.get::<_, Option<i32>>("rejected_signals").unwrap_or(0);
        }

        let profit_total: f64 = profits.iter().sum();
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
        let expectancy = (win_rate * avg_trade_profit) - (loss_rate * avg_trade_profit.abs()); // Corregido

        let total_gain: f64 = profits.iter().filter(|&&p| p > 0.0).sum();
        let total_loss: f64 = profits.iter().filter(|&&p| p < 0.0).map(|p| p.abs()).sum();
        let profit_factor = if total_loss > 0.0 {
            total_gain / total_loss
        } else {
            999.0
        }; // Valor alto para ganancias sin pérdidas

        let avg_monthly_profit = mean(profits);
        let std_monthly_profit = stddev(profits, avg_monthly_profit);
        let max_profit_month = profits.iter().cloned().fold(f64::MIN, f64::max);
        let min_profit_month = profits.iter().cloned().fold(f64::MAX, f64::min);
        let neg_months = profits.iter().filter(|&&p| p < 0.0).count();

        let (_, trailing_stop_positive, trailing_stop_positive_offset) = items[0];

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
            monthly: monthly_map,
        });
    }

    Ok(result)
}

fn generate_timeranges(start_date: &str, months: usize) -> Vec<String> {
    let mut ranges = vec![];
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d").unwrap();

    for i in 0..months {
        let from = start
            .with_day(1)
            .unwrap()
            .checked_add_months(Months::new(i as u32))
            .unwrap();
        let to = from.checked_add_months(Months::new(1)).unwrap();
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
    if data.is_empty() {
        return 0.0;
    }
    let variance = data
        .iter()
        .map(|value| {
            let diff = mean - *value;
            diff * diff
        })
        .sum::<f64>()
        / data.len() as f64;
    variance.sqrt()
}

pub fn export_summary_to_csv(
    data: &[StrategyGridRow],
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(filename)?;
    let mut wtr = Writer::from_writer(file);

    // ... (El resto de esta función puede permanecer igual, pero es buena idea que devuelva un Result)

    Ok(())
}
