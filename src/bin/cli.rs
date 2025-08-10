#![cfg(feature = "cli")]
use chrono::Local;
use clap::Parser;
use freqdash::{
    backtest::logic::export_summary_to_csv,
    backtest::logic::{GridQuery, get_grid_summary},
    config::init,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "BINANCE")]
    exchange: String,

    #[arg(short, long, default_value = "USDT")]
    currency: String,

    #[arg(short, long, default_value = "BTC")]
    pairlist: String,

    #[arg(short, long, default_value = "2024-01-01")]
    start_date: String,

    #[arg(short, long, default_value_t = 6)]
    months: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init().await;
    let args = Args::parse();

    let params = GridQuery {
        exchange: args.exchange,
        currency: args.currency,
        pairlist: args.pairlist,
        start_date: args.start_date,
        months: args.months,
    };

    println!("Ejecutando backtest con parámetros:");
    println!("Exchange: {}", params.exchange);
    println!("Moneda: {}", params.currency);
    println!("Pares: {}", params.pairlist);
    println!("Fecha inicio: {}", params.start_date);
    println!("Meses: {}", params.months);

    // Manejar el Result correctamente
    let mut summary = match get_grid_summary(&params).await {
        Ok(data) => {
            println!("✅ Consulta ejecutada exitosamente. {} estrategias encontradas.", data.len());
            data
        }
        Err(e) => {
            eprintln!("❌ Error ejecutando consulta: {}", e);
            return Err(e);
        }
    };

    // Ordenar por profit total (mayor a menor)
    summary.sort_by(|a, b| {
        b.total_profit
            .partial_cmp(&a.total_profit)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Mostrar top 5 en consola
    println!("\n🏆 Top 5 estrategias por profit total:");
    for (i, strategy) in summary.iter().take(5).enumerate() {
        println!(
            "{}. {} ({}) - Profit: {:.2}% | Trades: {} | Win Rate: {:.1}%",
            i + 1,
            strategy.strategy,
            strategy.timeframe,
            strategy.total_profit,
            strategy.total_trades,
            strategy.win_rate * 100.0
        );
    }

    // Exportar a CSV
    let now = Local::now();
    let filename = format!("reporte_backtest_{}.csv", now.format("%Y%m%d_%H%M%S"));
    export_summary_to_csv(&summary, &filename);
    
    println!("\n💾 Reporte exportado a: {}", filename);
    
    Ok(())
}