use chrono::Utc;

use crate::chunkroll_predictor::PredictionResult;

pub async fn make_sheep_api_call(
    auth_token: &str,
    prediction: &PredictionResult,
) -> eyre::Result<()> {
    let payload = SheepPayload {
        roll_number: 110,
        eta_date: prediction.average_chunkroll_date,
        range_start: prediction.lower_bound_chunkroll_date,
        range_end: prediction.upper_bound_chunkroll_date,
        method: "Kill 400k chaos dwarves for Larran's keys to obtain Dagon'hai robes".to_string(),
    };

    let response = reqwest::Client::new()
        .post("https://www.limpwurt.app/api/webhook/roll-estimate")
        .bearer_auth(auth_token)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let response_text = response.text().await?;

    println!("Sent Sheep API call. Response: {}", response_text);

    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SheepPayload {
    roll_number: u64,
    eta_date: chrono::DateTime<Utc>,
    range_start: chrono::DateTime<Utc>,
    range_end: chrono::DateTime<Utc>,
    method: String,
}
