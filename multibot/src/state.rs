use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    User,
    AI,
    Weather,
    Time,
    Define,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub message: String,
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    pub timestamp: String,
}

impl Message {
    pub fn new(sender: String, message: String, msg_type: MessageType) -> Self {
        Self {
            id: uuid_simple(),
            sender,
            message,
            msg_type,
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
        }
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}-{}", duration.as_millis(), rand::random::<u32>())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugLogEntry {
    pub timestamp: String,
    pub message: String,
}

impl DebugLogEntry {
    pub fn new(message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().format("%H:%M:%S").to_string(),
            message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub name: String,
    pub creator: String,
    pub greeting: String,
    pub system_prompt: String,
    pub traits: Vec<String>,
    pub speaking_style: SpeakingStyle,
    pub emotional_responses: EmotionalResponses,
    pub catchphrases: Vec<String>,
    pub catchphrase_chance: f32,
    pub idle_messages: Vec<String>,
    pub idle_timeout_seconds: u64,
    pub conversation_history_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingStyle {
    pub use_emoji: bool,
    pub casual_speech: bool,
    pub exclamation_heavy: bool,
    pub max_response_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionalResponses {
    pub excited_keywords: Vec<String>,
    pub excited_response: String,
    pub sad_keywords: Vec<String>,
    pub sad_response: String,
    pub confused_keywords: Vec<String>,
    pub confused_response: String,
    pub angry_keywords: Vec<String>,
    pub angry_response: String,
    pub love_keywords: Vec<String>,
    pub love_response: String,
}

impl Personality {
    pub fn load(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn check_emotions(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();

        let emotion_map = [
            (
                "excited",
                &self.emotional_responses.excited_keywords,
                &self.emotional_responses.excited_response,
            ),
            (
                "sad",
                &self.emotional_responses.sad_keywords,
                &self.emotional_responses.sad_response,
            ),
            (
                "confused",
                &self.emotional_responses.confused_keywords,
                &self.emotional_responses.confused_response,
            ),
            (
                "angry",
                &self.emotional_responses.angry_keywords,
                &self.emotional_responses.angry_response,
            ),
            (
                "love",
                &self.emotional_responses.love_keywords,
                &self.emotional_responses.love_response,
            ),
        ];

        for (_, keywords, response) in &emotion_map {
            for keyword in keywords.iter() {
                if input_lower.contains(&keyword.to_lowercase()) {
                    return Some(response.to_string());
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn maybe_catchphrase(&self) -> Option<String> {
        if self.catchphrases.is_empty() {
            return None;
        }
        if rand::random::<f32>() < self.catchphrase_chance {
            Some(self.catchphrases[rand::random::<usize>() % self.catchphrases.len()].clone())
        } else {
            None
        }
    }

    pub fn random_catchphrase(&self) -> Option<String> {
        if self.catchphrases.is_empty() {
            return None;
        }
        Some(self.catchphrases[rand::random::<usize>() % self.catchphrases.len()].clone())
    }

    pub fn random_idle_message(&self) -> String {
        if self.idle_messages.is_empty() {
            "...".to_string()
        } else {
            self.idle_messages[rand::random::<usize>() % self.idle_messages.len()].clone()
        }
    }
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            name: "MultiBot".to_string(),
            creator: "".to_string(),
            greeting: "Hello! I'm MultiBot. How can I help you?".to_string(),
            system_prompt: "You are a helpful AI assistant.".to_string(),
            traits: vec![],
            speaking_style: SpeakingStyle::default(),
            emotional_responses: EmotionalResponses::default(),
            catchphrases: vec![],
            catchphrase_chance: 0.1,
            idle_messages: vec!["...".to_string()],
            idle_timeout_seconds: 20,
            conversation_history_limit: 10,
        }
    }
}

impl Default for SpeakingStyle {
    fn default() -> Self {
        Self {
            use_emoji: true,
            casual_speech: true,
            exclamation_heavy: true,
            max_response_length: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openrouter: OpenRouterConfig,
    pub app: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    pub models: Vec<String>,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_secs: u64,
    #[serde(default = "default_timeout")]
    pub request_timeout_secs: u64,
}

fn default_retry_delay() -> u64 {
    2
}
fn default_timeout() -> u64 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub site_url: String,
    pub site_name: String,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
}

fn default_idle_timeout() -> u64 {
    20
}
fn default_history_limit() -> usize {
    10
}

impl Config {
    pub fn load(path: &str) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    #[allow(dead_code)]
    pub fn load_from_app_data() -> Option<Self> {
        if let Some(app_dir) = dirs::data_dir() {
            let config_path = app_dir.join("com.multibot.app").join("user_settings.json");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path).ok()?;
                let user_settings: UserSettings = serde_json::from_str(&content).ok()?;
                let mut config = Config::default();
                config.openrouter.api_key = user_settings.api_key;
                return Some(config);
            }
        }
        None
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openrouter: OpenRouterConfig {
                api_key: String::new(),
                models: vec![
                    "stepfun/step-3.5-flash:free".to_string(),
                    "nvidia/nemotron-3-nano-30b-a3b:free".to_string(),
                    "meta-llama/llama-3.3-70b-instruct:free".to_string(),
                ],
                retry_delay_secs: 2,
                request_timeout_secs: 60,
            },
            app: AppConfig {
                site_url: "https://yoursite.com".to_string(),
                site_name: "MultiBot".to_string(),
                idle_timeout_secs: 20,
                history_limit: 10,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub api_key: String,
    pub personality: Personality,
    #[serde(rename = "isConfigured")]
    pub is_configured: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            personality: Personality::default(),
            is_configured: false,
        }
    }
}

impl UserSettings {
    pub fn load() -> Self {
        if let Some(app_dir) = dirs::data_dir() {
            let config_path = app_dir.join("com.multibot.app").join("user_settings.json");
            if config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&config_path) {
                    if let Ok(settings) = serde_json::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let app_dir = dirs::data_dir()
            .ok_or("Could not find app data directory")?
            .join("com.multibot.app");

        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

        let config_path = app_dir.join("user_settings.json");
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(config_path, json).map_err(|e| e.to_string())
    }
}
