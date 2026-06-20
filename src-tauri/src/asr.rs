use tauri::AppHandle;

#[tauri::command]
pub fn transcribe(app: &AppHandle, wav_bytes: &[u8]) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        transcribe_macos(app, wav_bytes)
    }
    #[cfg(target_os = "windows")]
    {
        transcribe_api(app, wav_bytes)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = app;
        let _ = wav_bytes;
        Err("Unsupported platform".into())
    }
}

#[cfg(target_os = "macos")]
fn transcribe_macos(app: &AppHandle, wav_bytes: &[u8]) -> Result<String, String> {
    use std::io::Write;
    use tauri::Manager;

    // Write WAV to temp file
    let wav_path = std::env::temp_dir().join(format!("s2w_{}.wav", std::process::id()));
    let mut f = std::fs::File::create(&wav_path)
        .map_err(|e| format!("Create temp file: {}", e))?;
    f.write_all(wav_bytes)
        .map_err(|e| format!("Write temp file: {}", e))?;
    drop(f);

    let wav_path_str = wav_path.to_string_lossy().to_string();

    // Get STT helper path from app resource dir
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Resource dir: {}", e))?;
    let stt_bin = resource_dir.join("bin/s2w_stt");

    // Fallback: try local bin dir (for dev mode)
    let stt_bin = if stt_bin.exists() {
        stt_bin
    } else {
        let local_bin = std::env::current_exe()
            .map(|p| p.parent().map(|d| d.join("../bin/s2w_stt")).unwrap_or(stt_bin.clone()))
            .unwrap_or(stt_bin.clone());
        if local_bin.exists() {
            local_bin
        } else {
            // Last fallback: check src-tauri/bin/
            let dev_bin = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bin/s2w_stt");
            dev_bin
        }
    };

    let lang = crate::store::get_config_inner(app, "lang");
    let lang_id = match lang.as_str() {
        "en" => "en-US",
        "zh" => "zh-CN",
        _ => "zh-CN",
    };

    log::info!("Calling SFSpeechRecognizer: {:?} {} {}", stt_bin, wav_path_str, lang_id);

    let output = std::process::Command::new(&stt_bin)
        .arg(&wav_path_str)
        .arg(lang_id)
        .output()
        .map_err(|e| format!("SFSpeechRecognizer execution failed: {}", e))?;

    // Cleanup temp file if helper didn't
    let _ = std::fs::remove_file(&wav_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SFSpeechRecognizer error: {}", stderr.trim()));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    log::info!("SFSpeechRecognizer result: {:?}", text);
    Ok(text)
}

#[cfg(target_os = "windows")]
fn transcribe_api(app: &AppHandle, wav_bytes: &[u8]) -> Result<String, String> {
    let api_key = crate::store::get_config_inner(app, "api_key");
    let api_key = if api_key.is_empty() {
        std::env::var("GROQ_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .or_else(|_| std::env::var("ASR_API_KEY"))
            .unwrap_or_default()
    } else {
        api_key
    };
    if api_key.is_empty() {
        return Err("API key not configured. Set it in Settings or env var GROQ_API_KEY/OPENAI_API_KEY".into());
    }

    let base_url = crate::store::get_config_inner(app, "base_url");
    let base_url = if base_url.is_empty() {
        "https://api.groq.com/openai".to_string()
    } else {
        base_url.trim_end_matches('/').to_string()
    };

    let model = crate::store::get_config_inner(app, "model");
    let model = if model.is_empty() { "whisper-large-v3-turbo".to_string() } else { model };

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
