use crate::{api, calculator, define, memory::{self, UserMemory}, state::{Config, DebugLogEntry, Message, MessageType, Personality, UserSettings}, time};
use rand::random;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub config: Mutex<Config>,
    pub personality: Mutex<Personality>,
    pub conversation_history: Mutex<Vec<api::Message>>,
    pub debug_logs: Mutex<Vec<DebugLogEntry>>,
    pub memory: Mutex<UserMemory>,
}

impl AppState {
    pub fn new() -> Self {
        let user_settings = UserSettings::load();
        let is_configured = user_settings.is_configured;
        
        let config = if is_configured && !user_settings.api_key.is_empty() {
            let mut cfg = Config::default();
            cfg.openrouter.api_key = user_settings.api_key.clone();
            cfg
        } else {
            Config::load("config.json").unwrap_or_default()
        };
        
        let personality = if is_configured {
            user_settings.personality
        } else {
            match Personality::load("personality.json") {
                Some(p) => {
                    println!("[MultiBot] Loaded personality: {}", p.name);
                    p
                }
                None => {
                    println!("[MultiBot] Failed to load personality.json");
                    Personality::default()
                }
            }
        };
        
        let memory = UserMemory::load("user_memory.json");
        println!("[MultiBot] Loaded user memory with {} facts", memory.facts.len());
        println!("[MultiBot] Configured: {}", is_configured);
        
        Self {
            config: Mutex::new(config),
            personality: Mutex::new(personality),
            conversation_history: Mutex::new(vec![]),
            debug_logs: Mutex::new(vec![]),
            memory: Mutex::new(memory),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn process_message(
    user_input: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<Message>, String> {
    let mut result: Vec<Message> = vec![];

    let config = state.config.lock().unwrap().clone();
    let personality = state.personality.lock().unwrap().clone();

    if crate::weather::is_weather_command(&user_input) {
        if let Some(city) = crate::weather::extract_city(&user_input) {
            state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Weather: {}", city)));
            
            let is_explicit = crate::weather::is_explicit_weather_command(&user_input);
            
            match crate::weather::get_weather(&city).await {
                Ok(response) => {
                    if is_explicit {
                        result.push(Message::new("Weather".to_string(), response, MessageType::Weather));
                    } else {
                        match crate::weather::get_weather_for_ai(&city).await {
                            Ok((location, weather_summary)) => {
                                let ai_prompt = format!(
                                    "The current weather in {} is: {}. Respond in character as {} and naturally tell the user about the weather in a casual, conversational way. Keep it short and fun!",
                                    location, weather_summary, personality.name
                                );
                                
                                let messages = vec![
                                    api::Message::system(personality.system_prompt.clone()),
                                    api::Message::user(ai_prompt),
                                ];
                                
                                match api::try_models(
                                    &config.openrouter.api_key,
                                    &config.openrouter.models,
                                    messages,
                                    config.openrouter.request_timeout_secs,
                                    config.openrouter.retry_delay_secs,
                                ).await {
                                    Ok(ai_response) => {
                                        state.debug_logs.lock().unwrap().push(DebugLogEntry::new("AI weather response received".to_string()));
                                        result.push(Message::new(personality.name.clone(), ai_response, MessageType::AI));
                                    }
                                    Err(e) => {
                                        state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("AI weather error: {}", e)));
                                        result.push(Message::new("Weather".to_string(), response, MessageType::Weather));
                                    }
                                }
                            }
                            Err(e) => {
                                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Weather AI parse error: {}", e)));
                                result.push(Message::new("Weather".to_string(), response, MessageType::Weather));
                            }
                        }
                    }
                }
                Err(_) => {
                    result.push(Message::new(
                        "Weather".to_string(),
                        format!("Could not find weather for '{}'. Try being more specific.", city),
                        MessageType::Weather,
                    ));
                }
            }
        } else {
            result.push(Message::new(
                "Weather".to_string(),
                "Please specify a city. Example: 'What's the weather in Tokyo?'".to_string(),
                MessageType::Weather,
            ));
        }
    } else if time::is_time_command(&user_input) {
        state.debug_logs.lock().unwrap().push(DebugLogEntry::new("Time command detected".to_string()));
        
        let is_explicit = time::is_explicit_time_command(&user_input);
        
        match time::get_time().await {
            Ok(time_response) => {
                if is_explicit {
                    result.push(Message::new("Time".to_string(), time_response, MessageType::Time));
                } else {
                    let time_for_ai = time::get_time_for_ai();
                    let ai_prompt = format!(
                        "The current time is {}. Respond in character as {} and naturally tell the user what time it is in a casual, conversational way.",
                        time_for_ai, personality.name
                    );
                    
                    let messages = vec![
                        api::Message::system(personality.system_prompt.clone()),
                        api::Message::user(ai_prompt),
                    ];
                    
                    match api::try_models(
                        &config.openrouter.api_key,
                        &config.openrouter.models,
                        messages,
                        config.openrouter.request_timeout_secs,
                        config.openrouter.retry_delay_secs,
                    ).await {
                        Ok(ai_response) => {
                            state.debug_logs.lock().unwrap().push(DebugLogEntry::new("AI time response received".to_string()));
                            result.push(Message::new(personality.name.clone(), ai_response, MessageType::AI));
                        }
                        Err(e) => {
                            state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("AI time error: {}", e)));
                            result.push(Message::new("Time".to_string(), time_response, MessageType::Time));
                        }
                    }
                }
            }
            Err(e) => {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Time error: {}", e)));
                result.push(Message::new(
                    "Time".to_string(),
                    "Could not fetch time. Please try again.".to_string(),
                    MessageType::Time,
                ));
            }
        }
    } else if define::is_define_command(&user_input) {
        if let Some(word) = define::extract_word(&user_input) {
            state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Define: {}", word)));
            
            match define::get_definition(&word).await {
                Ok(response) => {
                    result.push(Message::new("Dictionary".to_string(), response, MessageType::Define));
                }
                Err(e) => {
                    result.push(Message::new(
                        "Dictionary".to_string(),
                        e,
                        MessageType::Define,
                    ));
                }
            }
        } else {
            result.push(Message::new(
                "Dictionary".to_string(),
                "Please specify a word. Example: '/define happy'".to_string(),
                MessageType::Define,
            ));
        }
    } else if calculator::is_calculator_command(&user_input) {
        state.debug_logs.lock().unwrap().push(DebugLogEntry::new("Calculator command detected".to_string()));
        
        if let Err(e) = calculator::open_calculator(app_handle.clone()).await {
            state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Calculator error: {}", e)));
            result.push(Message::new(
                "System".to_string(),
                "Could not open calculator. Please try again.".to_string(),
                MessageType::System,
            ));
        } else {
            result.push(Message::new(
                "System".to_string(),
                "Calculator opened! Use it whenever you need. Close it when done.".to_string(),
                MessageType::System,
            ));
        }
    } else if memory::is_remember_command(&user_input) {
        if let Some(fact) = memory::extract_remember_fact(&user_input) {
            state.memory.lock().unwrap().add_fact(fact.clone(), 5, "explicit");
            state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Remembered: {}", fact)));
            if let Err(e) = state.memory.lock().unwrap().save("user_memory.json") {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Memory save error: {}", e)));
            }
            result.push(Message::new(
                personality.name.clone(),
                format!("Gotcha! I'll remember that~ {}", personality.random_catchphrase().unwrap_or_default()),
                MessageType::AI,
            ));
        } else {
            result.push(Message::new(
                personality.name.clone(),
                "What should I remember? Try '/remember my cat's name is Luna'".to_string(),
                MessageType::AI,
            ));
        }
    } else if memory::is_forget_command(&user_input) {
        if let Some(fact) = memory::extract_forget_fact(&user_input) {
            let removed = state.memory.lock().unwrap().remove_fact(&fact);
            if let Err(e) = state.memory.lock().unwrap().save("user_memory.json") {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Memory save error: {}", e)));
            }
            if removed {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Forgot: {}", fact)));
                result.push(Message::new(
                    personality.name.clone(),
                    "Okay, I've forgotten that~".to_string(),
                    MessageType::AI,
                ));
            } else {
                result.push(Message::new(
                    personality.name.clone(),
                    "I don't think I have that stored... (・_・)".to_string(),
                    MessageType::AI,
                ));
            }
        } else {
            result.push(Message::new(
                personality.name.clone(),
                "What should I forget? Try '/forget my cat's name'".to_string(),
                MessageType::AI,
            ));
        }
    } else if memory::is_memories_command(&user_input) {
        let facts = state.memory.lock().unwrap().get_all_facts();
        let user_name = state.memory.lock().unwrap().user_name.clone();
        let mut response = String::new();
        if let Some(name) = user_name {
            response.push_str(&format!("Your name: {}\n\n", name));
        }
        if facts.is_empty() {
            response.push_str("No memories stored yet~ (⊙_⊙)");
        } else {
            response.push_str("Here's what I remember:\n");
            for (i, fact) in facts.iter().enumerate() {
                response.push_str(&format!("{}. {}\n", i + 1, fact));
            }
        }
        result.push(Message::new(
            "Memory".to_string(),
            response,
            MessageType::System,
        ));
    } else {
        if let Some(emotion) = personality.check_emotions(&user_input) {
            result.push(Message::new(personality.name.clone(), emotion, MessageType::AI));
        }

        if let Some((fact, importance)) = memory::detect_implicit_facts(&user_input) {
            {
                let mut mem = state.memory.lock().unwrap();
                mem.add_fact(fact.clone(), importance, "implicit");
                
                if fact.starts_with("user's name is ") {
                    let name = fact.strip_prefix("user's name is ").unwrap_or(&fact);
                    mem.user_name = Some(name.to_string());
                }
            }
            state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Auto-remembered: {}", fact)));
            if let Err(e) = state.memory.lock().unwrap().save("user_memory.json") {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Memory save error: {}", e)));
            }
        }

        let memory_context = state.memory.lock().unwrap().get_context_for_ai(&user_input);
        
        let system_prompt = if memory_context.is_empty() {
            personality.system_prompt.clone()
        } else {
            format!("{} {}", memory_context, personality.system_prompt)
        };

        let mut messages = vec![api::Message::system(system_prompt)];
        
        for hist in state.conversation_history.lock().unwrap().iter() {
            messages.push(hist.clone());
        }
        messages.push(api::Message::user(user_input.clone()));

        state.debug_logs.lock().unwrap().push(DebugLogEntry::new("Sending request...".to_string()));

        match api::try_models(
            &config.openrouter.api_key,
            &config.openrouter.models,
            messages,
            config.openrouter.request_timeout_secs,
            config.openrouter.retry_delay_secs,
        ).await {
            Ok(response) => {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new("Response received".to_string()));
                
                state.conversation_history.lock().unwrap().push(api::Message::user(user_input));
                state.conversation_history.lock().unwrap().push(api::Message::assistant(response.clone()));
                
                let limit = personality.conversation_history_limit * 2;
                let mut history = state.conversation_history.lock().unwrap();
                while history.len() > limit {
                    history.remove(0);
                }

                result.push(Message::new(personality.name.clone(), response, MessageType::AI));
            }
            Err(e) => {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("Error: {}", e)));
                result.push(Message::new("AI".to_string(), e, MessageType::System));
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_personality(state: State<'_, AppState>) -> Personality {
    state.personality.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_debug_logs(state: State<'_, AppState>) -> Vec<DebugLogEntry> {
    state.debug_logs.lock().unwrap().clone()
}

#[tauri::command]
pub async fn get_idle_message(state: State<'_, AppState>) -> Result<Message, String> {
    let personality = state.personality.lock().unwrap().clone();
    let config = state.config.lock().unwrap().clone();
    
    let use_ai = random::<f32>() < 0.30;
    
    if use_ai && !config.openrouter.api_key.is_empty() {
        state.debug_logs.lock().unwrap().push(DebugLogEntry::new("Generating AI idle message".to_string()));
        
        let history: Vec<api::Message> = state.conversation_history
            .lock()
            .unwrap()
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect();
        
        match api::generate_idle_message(
            &config.openrouter.api_key,
            &config.openrouter.models[0],
            history,
            &personality.system_prompt,
            config.openrouter.request_timeout_secs,
        ).await {
            Ok(response) => {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new("AI idle message generated".to_string()));
                return Ok(Message::new(personality.name.clone(), response, MessageType::AI));
            }
            Err(e) => {
                state.debug_logs.lock().unwrap().push(DebugLogEntry::new(format!("AI idle failed: {}", e)));
            }
        }
    }
    
    state.debug_logs.lock().unwrap().push(DebugLogEntry::new("Static idle message sent".to_string()));
    Ok(Message::new(personality.name.clone(), personality.random_idle_message(), MessageType::AI))
}

#[tauri::command]
pub fn is_configured() -> bool {
    let settings = UserSettings::load();
    settings.is_configured && !settings.api_key.is_empty()
}

#[tauri::command]
pub fn load_settings() -> UserSettings {
    UserSettings::load()
}

#[tauri::command]
pub async fn save_settings(
    api_key: String,
    personality: Personality,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let settings = UserSettings {
        api_key: api_key.clone(),
        personality: personality.clone(),
        is_configured: true,
    };
    
    settings.save()?;
    
    {
        let mut config = state.config.lock().unwrap();
        config.openrouter.api_key = api_key;
    }
    
    {
        let mut pers = state.personality.lock().unwrap();
        *pers = personality;
    }
    
    Ok(true)
}
