mod tracker;
mod parser;
mod exporter;

use colored::*;
use std::env;
use std::process;

fn print_banner() {
    println!("{}", r#"
 ███████╗ ██████╗ ██╗      █████╗ ███╗   ██╗ █████╗
 ██╔════╝██╔═══██╗██║     ██╔══██╗████╗  ██║██╔══██╗
 ███████╗██║   ██║██║     ███████║██╔██╗ ██║███████║
 ╚════██║██║   ██║██║     ██╔══██║██║╚██╗██║██╔══██║
 ███████║╚██████╔╝███████╗██║  ██║██║ ╚████║██║  ██║
 ╚══════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝
"#.bright_cyan().bold());

    println!("{}", "  ██╗    ██╗ █████╗ ██╗     ██╗     ███████╗████████╗".bright_green());
    println!("{}", "  ██║    ██║██╔══██╗██║     ██║     ██╔════╝╚══██╔══╝".bright_green());
    println!("{}", "  ██║ █╗ ██║███████║██║     ██║     █████╗     ██║   ".bright_green());
    println!("{}", "  ██║███╗██║██╔══██║██║     ██║     ██╔══╝     ██║   ".bright_green());
    println!("{}", "  ╚███╔███╔╝██║  ██║███████╗███████╗███████╗   ██║   ".bright_green());
    println!("{}", "   ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝   ╚═╝   ".bright_green());

    println!();
    println!(
        "  {}  {}",
        "◎".bright_yellow(),
        "Real-time SOL Transaction Monitor".white().bold()
    );
    println!(
        "  {}  {}",
        "◎".bright_yellow(),
        "by roybeey.com | Rust Edition".truecolor(150, 150, 150)
    );
    println!("{}", "─".repeat(60).truecolor(60, 60, 60));
    println!();
}

fn print_usage() {
    println!("{}", "USAGE:".bright_yellow().bold());
    println!(
        "  {} {} {}",
        "solana-wallet-tracker".bright_cyan(),
        "<WALLET_ADDRESS>".white(),
        "[OPTIONS]".truecolor(150, 150, 150)
    );
    println!();
    println!("{}", "OPTIONS:".bright_yellow().bold());
    println!("  {}  {}", "--limit <N>".bright_green(),   "Number of transactions to fetch (default: 20)");
    println!("  {}  {}", "--output <FILE>".bright_green(),"CSV output path (default: output/transactions.csv)");
    println!("  {}  {}", "--rpc <URL>".bright_green(),    "Custom RPC URL (default: mainnet-beta public)");
    println!("  {}  {}", "--help".bright_green(),         "Show this help message");
    println!();
    println!("{}", "EXAMPLES:".bright_yellow().bold());
    println!(
        "  {} {}",
        "solana-wallet-tracker".bright_cyan(),
        "vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg".white()
    );
    println!(
        "  {} {} {}",
        "solana-wallet-tracker".bright_cyan(),
        "vines1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg".white(),
        "--limit 50 --output output/my_wallet.csv".truecolor(150, 150, 150)
    );
    println!();
}

struct Args {
    wallet: String,
    limit: usize,
    output: String,
    rpc_url: String,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = env::args().collect();

    if raw.len() < 2 || raw.iter().any(|a| a == "--help" || a == "-h") {
        print_usage();
        process::exit(0);
    }

    let wallet = raw[1].clone();

    if wallet.len() < 32 || wallet.len() > 44 {
        return Err(format!(
            "Invalid wallet address: '{}' (expected 32-44 base58 chars)",
            wallet
        ));
    }

    let mut limit   = 20usize;
    let mut output  = "output/transactions.csv".to_string();
    let mut rpc_url = "https://api.mainnet-beta.solana.com".to_string();

    let mut i = 2;
    while i < raw.len() {
        match raw[i].as_str() {
            "--limit" => {
                i += 1;
                limit = raw.get(i)
                    .ok_or("--limit requires a value")?
                    .parse::<usize>()
                    .map_err(|_| "--limit must be a positive integer")?;
            }
            "--output" => {
                i += 1;
                output = raw.get(i).ok_or("--output requires a value")?.clone();
            }
            "--rpc" => {
                i += 1;
                rpc_url = raw.get(i).ok_or("--rpc requires a value")?.clone();
            }
            unknown => return Err(format!("Unknown argument: '{}'", unknown)),
        }
        i += 1;
    }

    Ok(Args { wallet, limit, output, rpc_url })
}

#[tokio::main]
async fn main() {
    print_banner();

    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{} {}", "  ERROR:".bright_red().bold(), e.white());
            eprintln!("  Run {} for usage.", "solana-wallet-tracker --help".bright_cyan());
            process::exit(1);
        }
    };

    println!("{}", "  CONFIGURATION".bright_yellow().bold());
    println!("  {} {}", "Wallet :".truecolor(120, 120, 120), args.wallet.bright_cyan());
    println!("  {} {}", "Limit  :".truecolor(120, 120, 120), args.limit.to_string().white());
    println!("  {} {}", "Output :".truecolor(120, 120, 120), args.output.white());
    println!("  {} {}", "RPC    :".truecolor(120, 120, 120), args.rpc_url.truecolor(150, 150, 150));
    println!("{}", "─".repeat(60).truecolor(60, 60, 60));
    println!();

    // ── Balance ──────────────────────────────────────────────────────────
    println!("  {} Fetching SOL balance...", "▶".bright_green());
    match tracker::get_balance(&args.wallet, &args.rpc_url).await {
        Ok(bal) => println!(
            "  {} Balance: {} SOL",
            "✔".bright_green(),
            format!("{:.6}", bal).bright_yellow().bold()
        ),
        Err(e) => eprintln!("  {} Failed to fetch balance: {}", "✘".bright_red(), e),
    }
    println!();

    // ── Signatures ───────────────────────────────────────────────────────
    println!(
        "  {} Fetching last {} transaction signatures...",
        "▶".bright_green(), args.limit
    );
    let signatures = match tracker::get_signatures(&args.wallet, args.limit, &args.rpc_url).await {
        Ok(s) => {
            println!("  {} Found {} signatures", "✔".bright_green(), s.len().to_string().white().bold());
            s
        }
        Err(e) => {
            eprintln!("  {} {}", "✘".bright_red(), e);
            process::exit(1);
        }
    };
    println!();

    // ── Parse transactions ───────────────────────────────────────────────
    println!("  {} Parsing transactions...", "▶".bright_green());

    let mut parsed_txs: Vec<parser::ParsedTransaction> = Vec::new();

    for (idx, sig) in signatures.iter().enumerate() {
        print!(
            "\r  {} [{}/{}] {}   ",
            "◈".bright_cyan(),
            idx + 1,
            signatures.len(),
            &sig[..20].truecolor(120, 120, 120)
        );

        match tracker::get_transaction(sig, &args.rpc_url).await {
            Ok(raw_tx) => {
                let parsed = parser::parse_transaction(sig, &raw_tx, &args.wallet);
                parsed_txs.push(parsed);
            }
            Err(e) => {
                eprintln!(
                    "\n  {} Skipping {}: {}",
                    "⚠".bright_yellow(), &sig[..20], e
                );
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    println!();
    println!(
        "  {} Parsed {} transactions successfully",
        "✔".bright_green(),
        parsed_txs.len().to_string().white().bold()
    );
    println!();

    // ── Summary + table ──────────────────────────────────────────────────
    parser::print_summary(&parsed_txs, &args.wallet);

    // ── Export CSV ───────────────────────────────────────────────────────
    println!("  {} Exporting to CSV: {}", "▶".bright_green(), args.output.white());
    match exporter::export_csv(&parsed_txs, &args.output) {
        Ok(path) => println!("  {} Saved → {}", "✔".bright_green(), path.bright_cyan()),
        Err(e)   => eprintln!("  {} Export failed: {}", "✘".bright_red(), e),
    }

    println!();
    println!("{}", "─".repeat(60).truecolor(60, 60, 60));
    println!(
        "  {} Run {} to visualise results.",
        "◎".bright_yellow(),
        "python charts/visualize.py".bright_cyan()
    );
    println!("{}", "─".repeat(60).truecolor(60, 60, 60));
    println!();
}