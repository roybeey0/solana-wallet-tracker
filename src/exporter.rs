use crate::parser::ParsedTransaction;
use std::fs;
use std::path::Path;

pub fn export_csv(txs: &[ParsedTransaction], output_path: &str) -> Result<String, String> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let mut wtr = csv::Writer::from_path(output_path)
        .map_err(|e| format!("Failed to create CSV file: {}", e))?;

    for tx in txs {
        wtr.serialize(tx)
            .map_err(|e| format!("Failed to write row: {}", e))?;
    }

    wtr.flush()
        .map_err(|e| format!("Failed to flush CSV: {}", e))?;

    Ok(output_path.to_string())
}