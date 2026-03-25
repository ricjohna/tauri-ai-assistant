use tauri::{AppHandle, Manager};

pub fn is_calculator_command(input: &str) -> bool {
    let input_lower = input.to_lowercase();
    input_lower.starts_with("/calculator") || input_lower.contains("open calculator")
}

#[tauri::command]
pub async fn open_calculator(app: AppHandle) -> Result<(), String> {
    let label = "calculator";
    
    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|e: tauri::Error| e.to_string())?;
        window.set_focus().map_err(|e: tauri::Error| e.to_string())?;
        return Ok(());
    }
    
    let url = tauri::WebviewUrl::App("calculator.html".into());
    
    tauri::WebviewWindowBuilder::new(&app, label, url)
        .title("Calculator")
        .inner_size(300.0, 450.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|e: tauri::Error| e.to_string())?;
    
    Ok(())
}
