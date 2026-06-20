use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tauri::{AppHandle, Emitter};

static RECORDING: AtomicBool = AtomicBool::new(false);
static STREAM: Mutex<Option<cpal::Stream>> = Mutex::new(None);
static AUDIO_BUFFER: Mutex<Vec<f32>> = Mutex::new(Vec::new());

#[tauri::command]
pub fn start_recording(app: AppHandle) -> Result<(), String> {
    if RECORDING.load(Ordering::SeqCst) {
        return Ok(());
    }

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;

    let supported_config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get input config: {}", e))?;

    let sample_rate: u32 = supported_config.sample_rate();
    let channels = supported_config.channels();
    log::info!("Recording started: {}Hz {}ch {:?}", sample_rate, channels, supported_config.sample_format());

    // Clear buffer
    AUDIO_BUFFER.lock().unwrap().clear();

    let config: cpal::StreamConfig = supported_config.into();

    let err_fn = |err: cpal::Error| {
        log::error!("Audio stream error: {}", err);
    };

    let stream = device
        .build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if RECORDING.load(Ordering::SeqCst) {
                    AUDIO_BUFFER.lock().unwrap().extend_from_slice(data);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Failed to build stream: {}", e))?;

    stream.play().map_err(|e| format!("Failed to start stream: {}", e))?;

    *STREAM.lock().unwrap() = Some(stream);
    RECORDING.store(true, Ordering::SeqCst);

    let _ = app.emit("recording-started", ());
    Ok(())
}

#[tauri::command]
pub fn stop_and_transcribe(app: AppHandle) -> Result<String, String> {
    RECORDING.store(false, Ordering::SeqCst);

    // Drop stream to stop recording
    *STREAM.lock().unwrap() = None;

    let _ = app.emit("recording-stopped", ());

    let samples = AUDIO_BUFFER.lock().unwrap().drain(..).collect::<Vec<f32>>();

    if samples.is_empty() {
        return Ok(String::new());
    }

    log::info!("Recording stopped: {} samples", samples.len());

    // Encode to WAV at 16kHz
    let wav_bytes = encode_wav(&samples, 16000)?;

    if wav_bytes.len() < 1024 {
        return Ok(String::new());
    }

    // Transcribe
    let text = crate::asr::transcribe(&app, &wav_bytes)?;

    if !text.is_empty() {
        let _ = app.emit("transcription-done", &text);
    }

    Ok(text)
}

fn encode_wav(samples: &[f32], target_sample_rate: u32) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: target_sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut buf, spec)
            .map_err(|e| format!("WAV writer create: {}", e))?;
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let val = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(val)
                .map_err(|e| format!("WAV write: {}", e))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("WAV finalize: {}", e))?;
    }

    Ok(buf.into_inner())
}
