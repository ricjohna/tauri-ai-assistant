use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: String) -> Self {
        Self { role: "system".to_string(), content }
    }
    pub fn user(content: String) -> Self {
        Self { role: "user".to_string(), content }
    }
    pub fn assistant(content: String) -> Self {
        Self { role: "assistant".to_string(), content }
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

pub async fn send_request(
    api_key: &str,
    model: &str,
    messages: Vec<Message>,
    timeout_secs: u64,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| e.to_string())?;

    let request = ChatRequest {
        model: model.to_string(),
        messages,
    };

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", "https://yoursite.com")
        .header("X-OpenRouter-Title", "MultiBot")
        .json(&request)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, body));
    }

    let chat_response: ChatResponse = response.json().await.map_err(|e| e.to_string())?;

    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response".to_string())
}

pub async fn try_models(
    api_key: &str,
    models: &[String],
    messages: Vec<Message>,
    timeout_secs: u64,
    retry_delay_secs: u64,
) -> Result<String, String> {
    let retry_codes = [429, 500, 502, 503, 504];

    for model in models {
        match send_request(api_key, model, messages.clone(), timeout_secs).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                if e.starts_with("HTTP 401") {
                    return Err("Authentication error. Please check your API key.".to_string());
                }
                if e.starts_with("HTTP ") {
                    if let Ok(code) = e[5..7].parse::<u16>() {
                        if retry_codes.contains(&code) {
                            tokio::time::sleep(std::time::Duration::from_secs(retry_delay_secs)).await;
                            continue;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(retry_delay_secs)).await;
            }
        }
    }

    Err("All models are currently busy. Please try again later.".to_string())
}

pub async fn generate_idle_message(
    api_key: &str,
    model: &str,
    conversation_history: Vec<Message>,
    system_prompt: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let context: String = conversation_history
        .iter()
        .rev()
        .take(3)
        .map(|m| format!("{}: {}", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let idle_prompt = format!(
        "You are {}. Based on this recent conversation:\n{}\n\nGenerate ONE short, playful idle message (max 20 characters) that shows you're waiting for the user to return. Keep it in character - cheerful, energetic, and fun. Just output the message, nothing else.",
        system_prompt, context
    );

    let messages = vec![Message::user(idle_prompt)];

    send_request(api_key, model, messages, timeout_secs).await
}
