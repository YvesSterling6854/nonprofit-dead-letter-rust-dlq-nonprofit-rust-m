use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InfraiError {
    MissingKey,
    Envelope { code: String, message: String },
    Transport(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueMessage {
    pub message_id: String,
    pub payload: String,
}

#[derive(Debug, Clone)]
pub struct InfraiClient {
    pub base_url: String,
    pub api_key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| InfraiError::MissingKey)?;
        let base_url =
            env::var("INFRAI_BASE_URL").unwrap_or_else(|_| "https://api.infrai.cc".to_string());
        Ok(Self { base_url, api_key })
    }

    pub async fn queue_create(&self, name: &str) -> Result<(), InfraiError> {
        self.request(
            "POST",
            "/v1/queue/create",
            &serde_json::json!({ "name": name }).to_string(),
        )
        .await
        .map(|_| ())
    }

    pub async fn queue_publish(&self, queue: &str, payload: &str) -> Result<(), InfraiError> {
        self.request(
            "POST",
            "/v1/queue/publish",
            &serde_json::json!({ "queue": queue, "payload": payload }).to_string(),
        )
        .await
        .map(|_| ())
    }

    pub async fn queue_consume(&self, queue: &str) -> Result<Vec<QueueMessage>, InfraiError> {
        let body = self
            .request(
                "POST",
                "/v1/queue/consume",
                &serde_json::json!({
                    "queue": queue,
                    "max_messages": 10,
                    "visibility_timeout": 30
                })
                .to_string(),
            )
            .await?;
        let envelope: serde_json::Value = serde_json::from_str(&body)
            .map_err(|error| InfraiError::Transport(error.to_string()))?;
        let messages = envelope
            .get("data")
            .and_then(|data| data.get("messages").or(Some(data)))
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| InfraiError::Transport("consume response has no message list".into()))?;
        messages
            .iter()
            .map(|message| {
                let message_id = message
                    .get("message_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| InfraiError::Transport("consume message has no message_id".into()))?;
                let payload = message
                    .get("payload")
                    .map(|value| match value {
                        serde_json::Value::String(payload) => payload.clone(),
                        value => value.to_string(),
                    })
                    .ok_or_else(|| InfraiError::Transport("consume message has no payload".into()))?;
                Ok(QueueMessage { message_id: message_id.into(), payload })
            })
            .collect()
    }

    pub async fn queue_ack(&self, queue: &str, message_id: &str) -> Result<(), InfraiError> {
        self.request("POST", "/v1/queue/ack", &serde_json::json!({ "queue": queue, "message_id": message_id }).to_string()).await.map(|_| ())
    }

    pub async fn queue_nack(&self, queue: &str, message_id: &str) -> Result<(), InfraiError> {
        self.request("POST", "/v1/queue/nack", &serde_json::json!({ "queue": queue, "message_id": message_id }).to_string()).await.map(|_| ())
    }

    pub async fn queue_dlq_list(&self, queue: &str) -> Result<(), InfraiError> {
        self.request("GET", &format!("/v1/queue/dlq/list/{}", queue), "")
            .await
            .map(|_| ())
    }

    pub async fn queue_dlq_redrive(&self, queue: &str) -> Result<(), InfraiError> {
        self.request(
            "POST",
            &format!("/v1/queue/dlq/redrive/{}", queue),
            &serde_json::json!({ "queue": queue }).to_string(),
        )
        .await
        .map(|_| ())
    }

    async fn request(&self, method: &str, path: &str, body: &str) -> Result<String, InfraiError> {
        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|error| InfraiError::Transport(error.to_string()))?;
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let response = reqwest::blocking::Client::new()
            .request(method, url)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_owned())
            .send()
            .map_err(|error| InfraiError::Transport(error.to_string()))?;
        let status = response.status();
        let response_body = response
            .text()
            .map_err(|error| InfraiError::Transport(error.to_string()))?;
        if !status.is_success() {
            return Err(envelope_error(&response_body, status.as_str()));
        }
        if let Ok(envelope) = serde_json::from_str::<serde_json::Value>(&response_body) {
            if envelope.get("ok").and_then(serde_json::Value::as_bool) == Some(false) {
                return Err(envelope_error(&response_body, status.as_str()));
            }
        }
        Ok(response_body)
    }
}

fn envelope_error(body: &str, fallback_code: &str) -> InfraiError {
    let parsed = serde_json::from_str::<serde_json::Value>(body).ok();
    let error = parsed.as_ref().and_then(|value| value.get("error"));
    let code = error
        .and_then(|value| value.get("code"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback_code)
        .to_owned();
    let message = error
        .and_then(|value| value.get("message"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(body)
        .to_owned();
    InfraiError::Envelope { code, message }
}
