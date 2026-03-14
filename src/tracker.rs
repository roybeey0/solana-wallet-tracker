use reqwest::Client;
use serde_json::{json, Value};

pub async fn get_balance(wallet: &str, rpc_url: &str) -> Result<f64, String> {
    let client = Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [wallet]
    });

    let res: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("JSON parse failed: {}", e))?;

    let lamports = res["result"]["value"]
        .as_u64()
        .ok_or("Could not parse balance")?;

    Ok(lamports as f64 / 1_000_000_000.0)
}

pub async fn get_signatures(
    wallet: &str,
    limit: usize,
    rpc_url: &str,
) -> Result<Vec<String>, String> {
    let client = Client::new();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignaturesForAddress",
        "params": [
            wallet,
            { "limit": limit }
        ]
    });

    let res: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("JSON parse failed: {}", e))?;

    let sigs = res["result"]
        .as_array()
        .ok_or("Could not parse signatures array")?
        .iter()
        .filter_map(|s| s["signature"].as_str().map(|v| v.to_string()))
        .collect();

    Ok(sigs)
}

pub async fn get_transaction(sig: &str, rpc_url: &str) -> Result<Value, String> {
    let client = Client::new();

    for encoding in &["jsonParsed", "json"] {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                sig,
                {
                    "encoding": encoding,
                    "maxSupportedTransactionVersion": 0,
                    "commitment": "confirmed"
                }
            ]
        });

        let res: Value = client
            .post(rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?
            .json()
            .await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        if !res["result"].is_null() {
            return Ok(res["result"].clone());
        }
    }

    Err(format!("Transaction not found (tried all encodings): {}", sig))
}