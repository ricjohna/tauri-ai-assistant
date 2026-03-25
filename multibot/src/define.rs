use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DictionaryResponse {
    word: String,
    phonetic: Option<String>,
    meanings: Vec<Meaning>,
}

#[derive(Debug, Deserialize)]
struct Meaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: String,
    definitions: Vec<Definition>,
}

#[derive(Debug, Deserialize)]
struct Definition {
    definition: String,
    example: Option<String>,
}

pub fn is_define_command(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    input_lower.starts_with("/define ") || input_lower.contains("define ")
}

pub fn extract_word(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    if input_lower.starts_with("/define ") {
        let word = input[8..].trim();
        if !word.is_empty() {
            return Some(word.to_string());
        }
    }
    
    if let Some(pos) = input_lower.find("define ") {
        let word = input[pos + 7..].trim();
        if !word.is_empty() {
            return Some(word.to_string());
        }
    }
    
    None
}

pub async fn get_definition(word: &str) -> Result<String, String> {
    let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", word);
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Could not find definition for '{}'", word));
    }

    let dict_data: Vec<DictionaryResponse> = response.json().await.map_err(|e| e.to_string())?;

    let entry = dict_data.first().ok_or_else(|| "No definition found".to_string())?;
    
    let mut result = format!("Definition of '{}'", entry.word);
    
    if let Some(phonetic) = &entry.phonetic {
        result.push_str(&format!(" {}", phonetic));
    }
    result.push('\n');

    if let Some(meaning) = entry.meanings.first() {
        result.push_str(&format!("\n({}) ", meaning.part_of_speech));
        
        if let Some(def) = meaning.definitions.first() {
            result.push_str(&def.definition);
            
            if let Some(example) = &def.example {
                result.push_str(&format!("\nExample: \"{}\"", example));
            }
        }
    }

    Ok(result)
}
