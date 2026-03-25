use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const MAX_FACTS: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMemory {
    pub user_name: Option<String>,
    pub preferences: HashMap<String, String>,
    pub facts: Vec<Fact>,
    #[serde(rename = "lastSeen")]
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub content: String,
    pub importance: u8,
    pub source: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "referenceCount")]
    pub reference_count: u32,
}

impl Default for UserMemory {
    fn default() -> Self {
        Self {
            user_name: None,
            preferences: HashMap::new(),
            facts: Vec::new(),
            last_seen: None,
        }
    }
}

impl UserMemory {
    pub fn load(path: &str) -> Self {
        let content = fs::read_to_string(path).ok();
        match content {
            Some(c) => serde_json::from_str(&c).unwrap_or_default(),
            None => Self::default(),
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn set_user_name(&mut self, name: String) {
        self.user_name = Some(name.clone());
        self.add_fact(format!("user's name is {}", name), 4, "explicit");
    }

    pub fn add_fact(&mut self, content: String, importance: u8, source: &str) {
        if self.facts.len() >= MAX_FACTS {
            self.facts.sort_by_key(|f| f.importance);
            if let Some(lowest) = self.facts.first() {
                if importance > lowest.importance {
                    self.facts.remove(0);
                } else {
                    return;
                }
            }
        }

        if let Some(existing) = self.facts.iter_mut().find(|f| f.content == content) {
            existing.reference_count += 1;
            return;
        }

        self.facts.push(Fact {
            content,
            importance: importance.min(5),
            source: source.to_string(),
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            reference_count: 1,
        });
    }

    pub fn remove_fact(&mut self, content: &str) -> bool {
        if let Some(pos) = self.facts.iter().position(|f| f.content.contains(content)) {
            self.facts.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_all_facts(&self) -> Vec<String> {
        self.facts.iter().map(|f| f.content.clone()).collect()
    }

    pub fn get_context_for_ai(&self, user_input: &str) -> String {
        let mut context = Vec::new();

        if let Some(name) = &self.user_name {
            context.push(format!("user's name is {}", name));
        }

        for fact in &self.facts {
            let input_lower = user_input.to_lowercase();
            let fact_lower = fact.content.to_lowercase();

            if fact_lower
                .split_whitespace()
                .any(|w| input_lower.contains(w))
                || fact_lower.contains("name") && input_lower.contains("name")
            {
                context.push(fact.content.clone());
            }
        }

        if context.is_empty() {
            String::new()
        } else {
            format!(
                "You know these things about the user: {}. ",
                context.join(", ")
            )
        }
    }

    #[allow(dead_code)]
    pub fn update_last_seen(&mut self) {
        self.last_seen = Some(chrono::Local::now().format("%Y-%m-%d %H:%M").to_string());
    }
}

pub fn is_remember_command(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.starts_with("/remember ") || lower.starts_with("remember ")
}

pub fn is_forget_command(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.starts_with("/forget ") || lower.starts_with("forget ")
}

pub fn is_memories_command(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower == "/memories" || lower == "memories"
}

pub fn extract_remember_fact(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    if lower.starts_with("/remember ") {
        let fact = input[10..].trim();
        if !fact.is_empty() {
            return Some(fact.to_string());
        }
    }

    if lower.starts_with("remember ") {
        let fact = input[9..].trim();
        if !fact.is_empty() {
            return Some(fact.to_string());
        }
    }

    None
}

pub fn extract_forget_fact(input: &str) -> Option<String> {
    let lower = input.to_lowercase();

    if lower.starts_with("/forget ") {
        let fact = input[8..].trim();
        if !fact.is_empty() {
            return Some(fact.to_string());
        }
    }

    if lower.starts_with("forget ") {
        let fact = input[7..].trim();
        if !fact.is_empty() {
            return Some(fact.to_string());
        }
    }

    None
}

pub fn detect_implicit_facts(input: &str) -> Option<(String, u8)> {
    let lower = input.to_lowercase();

    let patterns = [
        (r"(?i)^my name is (.+)$", 4, "user's name is {1}"),
        (r"(?i)^i'?m (.+)$", 4, "user's name is {1}"),
        (r"(?i)^call me (.+)$", 4, "user's name is {1}"),
        (r"(?i)i have (?:a |an )?(.+)$", 3, "user has {1}"),
        (r"(?i)i live in (.+)$", 3, "user lives in {1}"),
        (r"(?i)i work (?:at |as |in )(.+)$", 3, "user works {1}"),
        (
            r"(?i)i'?m (?:turning |going to be )?(\d+)",
            3,
            "user is {1} years old",
        ),
        (
            r"(?i)my birthday (?:is |on )(.+)$",
            4,
            "user's birthday is {1}",
        ),
        (r"(?i)i love (.+)$", 2, "user loves {1}"),
        (r"(?i)i like (.+)$", 2, "user likes {1}"),
        (r"(?i)i hate (.+)$", 2, "user hates {1}"),
        (r"(?i)i study (.+)$", 3, "user studies {1}"),
        (r"(?i)i play (.+)$", 2, "user plays {1}"),
        (r"(?i)i'?m from (.+)$", 3, "user is from {1}"),
    ];

    for (pattern, importance, template) in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(&lower) {
                if let Some(m) = caps.get(1) {
                    let content = m.as_str().trim().to_string();
                    if content.len() > 1 {
                        let fact = template.replace("{1}", &content);
                        return Some((fact, *importance));
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_remember_fact() {
        assert_eq!(
            extract_remember_fact("remember my cat's name is Mallows"),
            Some("my cat's name is Mallows".to_string())
        );
    }

    #[test]
    fn test_detect_implicit_facts() {
        assert_eq!(
            detect_implicit_facts("My name is Neon"),
            Some(("Neon".to_string(), 4))
        );
        assert_eq!(
            detect_implicit_facts("I have a cat"),
            Some(("a cat".to_string(), 3))
        );
    }
}
