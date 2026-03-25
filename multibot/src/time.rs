pub fn is_time_command(input: &str) -> bool {
    is_explicit_time_command(input) || is_conversational_time_command(input)
}

pub fn is_explicit_time_command(input: &str) -> bool {
    input.to_lowercase().starts_with("/time")
}

pub fn is_conversational_time_command(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    
    input_lower.contains("what time") || 
        input_lower.contains("what's the time") ||
        input_lower.contains("current time") ||
        input_lower.contains("tell me the time") ||
        input_lower.contains("got the time") ||
        input_lower.contains("do you know the time")
}

pub async fn get_time() -> Result<String, String> {
    let now = chrono::Local::now();
    Ok(format!(
        "Current Time (Philippines)\nDate: {}\nTime: {}\nTimezone: Asia/Manila (PHT)",
        now.format("%Y-%m-%d"),
        now.format("%H:%M:%S")
    ))
}

pub fn get_time_for_ai() -> String {
    let now = chrono::Local::now();
    now.format("%I:%M %p").to_string()
}
