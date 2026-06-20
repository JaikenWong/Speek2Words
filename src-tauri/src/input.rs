use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn type_text(app: AppHandle, text: String) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }

    log::info!("Typing text: {:?}", &text[..text.len().min(50)]);

    // Save current clipboard, set new text, simulate paste, restore clipboard
    let prev_clipboard = arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok());

    // Set clipboard
    {
        let mut clipboard =
            arboard::Clipboard::new().map_err(|e| format!("Clipboard init: {}", e))?;
        clipboard
            .set_text(&text)
            .map_err(|e| format!("Clipboard set: {}", e))?;
    }

    // Small delay for clipboard to settle
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Simulate Cmd+V (macOS) or Ctrl+V (Windows)
    simulate_paste()?;

    // Restore previous clipboard in background
    if let Some(prev) = prev_clipboard {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text(&prev);
            }
        });
    }

    let _ = app.emit("text-typed", ());
    Ok(())
}

#[cfg(target_os = "macos")]
fn simulate_paste() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo init: {}", e))?;
    enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("Meta press: {}", e))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("V click: {}", e))?;
    enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("Meta release: {}", e))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn simulate_paste() -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo init: {}", e))?;
    enigo.key(Key::Control, Direction::Press).map_err(|e| format!("Ctrl press: {}", e))?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("V click: {}", e))?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| format!("Ctrl release: {}", e))?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn simulate_paste() -> Result<(), String> {
    Err("Unsupported platform".into())
}
