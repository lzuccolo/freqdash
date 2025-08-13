use crate::backtest::model::GridQuery;

pub fn build_flat_sql(params: &GridQuery, timeranges: &[String]) -> String {
    let timerange_list = timeranges
        .iter()
        .map(|m| format!("'{}'", m))
        .collect::<Vec<String>>()
        .join(",");

    format!(
        r#"
        SELECT
            strategy, timeframe, minimal_roi, stoploss, max_open_trades,
            trailing_stop, trailing_stop_positive, trailing_stop_positive_offset,
            trailing_only_offset_is_reached, entry_pricing, exit_pricing,
            check_depth_of_market_enable,
            profit_total, total_trades, wins, winner_holding_avg_s,
            max_drawdown, rejected_signals, timerange
        FROM backtest
        WHERE
            exchange = '{exchange}' AND
            stake_currency = '{currency}' AND
            pairlist = '{pairlist}' AND
            timerange IN ({timeranges})
        "#,
        exchange = params.exchange.to_uppercase(),
        currency = params.currency.to_uppercase(),
        pairlist = params.pairlist.to_uppercase(),
        timeranges = timerange_list
    )
}
