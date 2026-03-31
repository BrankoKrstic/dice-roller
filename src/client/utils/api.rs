use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiErrorResponse {
    error: String,
}

pub async fn parse_error_response(response: reqwest::Response, fallback: &str) -> String {
    let status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| fallback.to_string());

    serde_json::from_str::<ApiErrorResponse>(&text)
        .map(|payload| payload.error)
        .unwrap_or_else(|_| {
            if text.trim().is_empty() {
                format!("{fallback} ({status})")
            } else {
                text
            }
        })
}
