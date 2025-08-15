// Añadir estas funciones optimizadas a src/backtest/logic.rs

use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Versión optimizada de get_grid_summary para grandes datasets
pub async fn get_grid_summary_chunked(params: &GridQuery) -> Result<Vec<StrategyGridRow>, Box<dyn std::error::Error>> {
    let timeranges = generate_timeranges(&params.start_date, params.months);
    
    // Dividir timeranges en chunks para procesamiento paralelo
    const CHUNK_SIZE: usize = 3; // Procesar 3 meses a la vez
    let chunks: Vec<_> = timeranges.chunks(CHUNK_SIZE).map(|c| c.to_vec()).collect();
    
    let client = AsyncDatabase::get_client().await;
    let results = Arc::new(Mutex::new(Vec::new()));
    
    // Procesar chunks en paralelo
    let futures = chunks.into_iter().map(|chunk| {
        let params = params.clone();
        let client = client.clone();
        let results = results.clone();
        
        async move {
            let sql = build_flat_sql(&params, &chunk);
            let client_lock = client.lock().await;
            
            match client_lock.query(&sql, &[]).await {
                Ok(rows) => {
                    let mut res = results.lock().await;
                    res.extend(rows);
                    Ok(())
                }
                Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>)
            }
        }
    });
    
    // Ejecutar hasta 3 consultas en paralelo
    let stream = stream::iter(futures).buffer_unordered(3);
    let outcomes: Vec<_> = stream.collect().await;
    
    // Verificar errores
    for outcome in outcomes {
        outcome?;
    }
    
    // Procesar resultados
    let rows = Arc::try_unwrap(results)
        .unwrap_or_else(|arc| (*arc.lock().unwrap()).clone())
        .into_inner();
    
    process_rows_to_grid(rows)
}

/// Procesa las filas de la BD en StrategyGridRow
fn process_rows_to_grid(rows: Vec<tokio_postgres::Row>) -> Result<Vec<StrategyGridRow>, Box<dyn std::error::Error>> {
    let mut grouped: HashMap<
        (String, String, String, String, i32, bool, String, String, bool, String, String, bool),
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

    let mut result = Vec::with_capacity(grouped.len());

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