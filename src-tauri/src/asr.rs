use tauri::AppHandle;

#[tauri::command]
pub fn transcribe(app: &AppHandle, wav_bytes: &[u8]) -> Result<String, String> {
    let api_key = crate::store::get_config_inner(app, "api_key");
    if api_key.is_empty() {
        return Err("API key not configured".into());
    }

    let base_url = crate::store::get_config_inner(app, "base_url");
    let base_url = if base_url.is_empty() {
        "https://api.minimaxi.com".to_string()
    } else {
        base_url.trim_end_matches('/').to_string()
    };

    let model = crate::store::get_config_inner(app, "model");
    let model = if model.is_empty() { "speech-01".to_string() } else { model };

    let lang = crate::store::get_config_inner(app, "lang");
    let lang = if lang.is_empty() { "zh".to_string() } else { lang };

    let url = format!("{}/v1/audio/transcriptions", base_url);
    log::info!("ASR request: {} model={}", url, model);

    let client = reqwest::blocking::Client::new();

    let part = reqwest::blocking::multipart::Part::bytes(wav_bytes.to_vec())
        .file_name("speech.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("MIME error: {}", e))?;

    let form = reqwest::blocking::multipart::Form::new()
        .part("file", part)
        .text("model", model)
        .text("language", lang);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .map_err(|e| format!("ASR request failed: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .map_err(|e| format!("ASR response read failed: {}", e))?;

    if !status.is_success() {
        log::error!("ASR error {}: {}", status, &body[..body.len().min(200)]);
        return Err(format!("ASR {} error: {}", status, &body[..body.len().min(200)]));
    }

    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON parse: {}", e))?;

    let text = json
        .get("text")
        .and_then(|v| v.as_str())
        .or_else(|| json.get("data").and_then(|d| d.get("text")).and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string();

    log::info!("ASR result: {:?}", text);
    Ok(text)
}
