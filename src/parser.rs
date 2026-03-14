use colored::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedTransaction {
    pub signature: String,
    pub block_time: String,
    pub slot: u64,
    pub tx_type: String,
    pub direction: String,
    pub amount_sol: f64,
    pub token_symbol: String,
    pub from_address: String,
    pub to_address: String,
    pub program_id: String,
    pub fee_sol: f64,
    pub status: String,
}

pub fn parse_transaction(sig: &str, raw: &Value, wallet: &str) -> ParsedTransaction {
    let slot = raw["slot"].as_u64().unwrap_or(0);

    let block_time = raw["blockTime"]
        .as_i64()
        .map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let fee_lamports = raw["meta"]["fee"].as_u64().unwrap_or(0);
    let fee_sol = fee_lamports as f64 / 1_000_000_000.0;

    let status = if raw["meta"]["err"].is_null() {
        "SUCCESS".to_string()
    } else {
        "FAILED".to_string()
    };

    let instructions = raw["transaction"]["message"]["instructions"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // ── SOL Transfer ─────────────────────────────────────────────────────
    for ix in &instructions {
        let program = ix["program"].as_str().unwrap_or("");
        let ix_type = ix["parsed"]["type"].as_str().unwrap_or("");

        if program == "system" && ix_type == "transfer" {
            let info = &ix["parsed"]["info"];
            let from = info["source"].as_str().unwrap_or("").to_string();
            let to   = info["destination"].as_str().unwrap_or("").to_string();
            let lamports = info["lamports"].as_u64().unwrap_or(0);
            let amount_sol = lamports as f64 / 1_000_000_000.0;

            let direction = if from == wallet {
                "OUT"
            } else if to == wallet {
                "IN"
            } else {
                "N/A"
            }
            .to_string();

            return ParsedTransaction {
                signature: sig.to_string(),
                block_time,
                slot,
                tx_type: "SOL_TRANSFER".to_string(),
                direction,
                amount_sol,
                token_symbol: "SOL".to_string(),
                from_address: from,
                to_address: to,
                program_id: "11111111111111111111111111111111".to_string(),
                fee_sol,
                status,
            };
        }
    }

    // ── SPL Token Transfer ───────────────────────────────────────────────
    for ix in &instructions {
        let program = ix["program"].as_str().unwrap_or("");
        let ix_type = ix["parsed"]["type"].as_str().unwrap_or("");

        if program == "spl-token"
            && (ix_type == "transfer" || ix_type == "transferChecked")
        {
            let info = &ix["parsed"]["info"];
            let from = info["source"].as_str().unwrap_or("").to_string();
            let to   = info["destination"].as_str().unwrap_or("").to_string();

            let amount_raw = info["tokenAmount"]["uiAmount"]
                .as_f64()
                .or_else(|| info["amount"].as_str().and_then(|a| a.parse().ok()))
                .unwrap_or(0.0);

            let mint = info["mint"].as_str().unwrap_or("unknown");
            let token_symbol = format!("{}…", &mint[..mint.len().min(8)]);

            let direction = if from == wallet {
                "OUT"
            } else if to == wallet {
                "IN"
            } else {
                "N/A"
            }
            .to_string();

            return ParsedTransaction {
                signature: sig.to_string(),
                block_time,
                slot,
                tx_type: "SPL_TRANSFER".to_string(),
                direction,
                amount_sol: amount_raw,
                token_symbol,
                from_address: from,
                to_address: to,
                program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                fee_sol,
                status,
            };
        }
    }

    // ── Program Interaction (fallback) ───────────────────────────────────
    let program_id = instructions
        .first()
        .and_then(|ix| ix["programId"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let program_label = if program_id.len() > 8 {
        format!("{}…", &program_id[..8])
    } else {
        program_id.clone()
    };

    ParsedTransaction {
        signature: sig.to_string(),
        block_time,
        slot,
        tx_type: "PROGRAM_INTERACTION".to_string(),
        direction: "N/A".to_string(),
        amount_sol: 0.0,
        token_symbol: program_label,
        from_address: wallet.to_string(),
        to_address: "contract".to_string(),
        program_id,
        fee_sol,
        status,
    }
}

pub fn print_summary(txs: &[ParsedTransaction], wallet: &str) {
    let total   = txs.len();
    let success = txs.iter().filter(|t| t.status == "SUCCESS").count();
    let failed  = total - success;

    let sol_in: f64 = txs
        .iter()
        .filter(|t| t.direction == "IN" && t.token_symbol == "SOL")
        .map(|t| t.amount_sol)
        .sum();

    let sol_out: f64 = txs
        .iter()
        .filter(|t| t.direction == "OUT" && t.token_symbol == "SOL")
        .map(|t| t.amount_sol)
        .sum();

    let total_fees: f64 = txs.iter().map(|t| t.fee_sol).sum();
    let net = sol_in - sol_out;

    let net_colored = if net >= 0.0 {
        format!("+{:.6}", net).bright_green().bold()
    } else {
        format!("{:.6}", net).bright_red().bold()
    };

    println!("{}", "  SUMMARY".bright_yellow().bold());
    println!(
        "  {} {}",
        "Wallet        :".truecolor(120, 120, 120),
        wallet.bright_cyan()
    );
    println!(
        "  {} {}",
        "Total Txns    :".truecolor(120, 120, 120),
        total.to_string().white().bold()
    );
    println!(
        "  {} {}  |  {}",
        "Status        :".truecolor(120, 120, 120),
        format!("{} SUCCESS", success).bright_green(),
        format!("{} FAILED", failed).bright_red(),
    );
    println!(
        "  {} {} SOL",
        "Total IN      :".truecolor(120, 120, 120),
        format!("{:.6}", sol_in).bright_green().bold()
    );
    println!(
        "  {} {} SOL",
        "Total OUT     :".truecolor(120, 120, 120),
        format!("{:.6}", sol_out).bright_red().bold()
    );
    println!(
        "  {} {} SOL",
        "Net           :".truecolor(120, 120, 120),
        net_colored
    );
    println!(
        "  {} {} SOL",
        "Total Fees    :".truecolor(120, 120, 120),
        format!("{:.6}", total_fees).bright_yellow()
    );
    println!("{}", "─".repeat(60).truecolor(60, 60, 60));
    println!();

    // ── Table ────────────────────────────────────────────────────────────
    println!(
        "  {:<20} {:<20} {:<8} {:<12} {:<6} {}",
        "TIME".truecolor(120, 120, 120),
        "SIGNATURE".truecolor(120, 120, 120),
        "TYPE".truecolor(120, 120, 120),
        "AMOUNT".truecolor(120, 120, 120),
        "DIR".truecolor(120, 120, 120),
        "STATUS".truecolor(120, 120, 120),
    );
    println!("  {}", "─".repeat(56).truecolor(60, 60, 60));

    for tx in txs.iter().take(20) {
        let time_short = if tx.block_time.len() >= 16 {
            &tx.block_time[..16]
        } else {
            &tx.block_time
        };

        let sig_short = format!("{}…", &tx.signature[..16]);

        let type_colored = match tx.tx_type.as_str() {
            "SOL_TRANSFER" => "SOL_TX ".bright_cyan(),
            "SPL_TRANSFER" => "SPL_TX ".bright_magenta(),
            _              => "PROG   ".truecolor(180, 180, 50).into(),
        };

        let amount_str = format!("{:.4} {}", tx.amount_sol, tx.token_symbol);

        let dir_colored = match tx.direction.as_str() {
            "IN"  => " IN ".on_bright_green().black().bold(),
            "OUT" => "OUT ".on_red().white().bold(),
            _     => "N/A ".on_truecolor(60, 60, 60).white().into(),
        };

        let status_colored = match tx.status.as_str() {
            "SUCCESS" => "✔".bright_green(),
            _         => "✘".bright_red(),
        };

        println!(
            "  {:<20} {:<20} {:<8} {:<12} {}  {}",
            time_short.truecolor(180, 180, 180),
            sig_short.truecolor(100, 180, 255),
            type_colored,
            amount_str.white(),
            dir_colored,
            status_colored,
        );
    }

    if txs.len() > 20 {
        println!(
            "  {} {} more transactions exported to CSV.",
            "…".truecolor(120, 120, 120),
            txs.len() - 20
        );
    }

    println!();
}