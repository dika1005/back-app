use reqwest::Client;
use serde_json::{json, Value};
use crate::AppState;

#[allow(dead_code)]
pub async fn create_midtrans_transaction(
    state: &AppState,
    order_id: &str,
    gross_amount: i64,
) -> Result<Value, String> {
    let client = Client::new();

    let body = json!({
        "transaction_details": {
            "order_id": order_id,
            "gross_amount": gross_amount
        },
        "credit_card": {
            "secure": true
        }
    });

    let url = format!("{}/snap/v1/transactions", state.midtrans_base_url);

    let res = client
        .post(&url)
        .basic_auth(&state.midtrans_server_key, Some(""))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data = res.json::<Value>().await.map_err(|e| e.to_string())?;
    Ok(data)
}
    