#!/usr/bin/env python3
"""Speek2Words — hold hotkey, speak, text lands at cursor."""
from __future__ import annotations

import io
import logging
import os
import sys
import threading
import time
from dataclasses import dataclass
from typing import Optional

import numpy as np
import requests
import sounddevice as sd
import soundfile as sf
from pynput import keyboard
from pynput.keyboard import Key, KeyCode

log = logging.getLogger("speek2words")

# ---------- config ----------
@dataclass
class Config:
    api_key: str
    base_url: str
    model: str
    hotkey: str
    sample_rate: int
    lang: str

    @classmethod
    def load(cls) -> "Config":
        try:
            from dotenv import load_dotenv
            load_dotenv()
        except Exception:
            pass
        return cls(
            api_key=os.getenv("MINIMAX_API_KEY", ""),
            base_url=os.getenv("MINIMAX_BASE_URL", "https://api.minimaxi.com").rstrip("/"),
            model=os.getenv("ASR_MODEL", "speech-01"),
            hotkey=os.getenv("HOTKEY", "Key.cmd_r"),
            sample_rate=int(os.getenv("SAMPLE_RATE", "16000")),
            lang=os.getenv("LANG", "zh"),
        )


# ---------- recorder ----------
class Recorder:
    def __init__(self, sample_rate: int):
        self.sr = sample_rate
        self._buf: list[np.ndarray] = []
        self._stream: Optional[sd.InputStream] = None
        self._lock = threading.Lock()
        self._active = False

    def start(self) -> None:
        with self._lock:
            if self._active:
                return
            self._buf = []
            self._active = True
            self._stream = sd.InputStream(
                samplerate=self.sr,
                channels=1,
                dtype="float32",
                callback=self._on_audio,
            )
            self._stream.start()
        log.info("rec started")

    def stop(self) -> bytes:
        with self._lock:
            self._active = False
            stream = self._stream
            self._stream = None
        if stream is not None:
            stream.stop()
            stream.close()
        audio = np.concatenate(self._buf, axis=0) if self._buf else np.zeros((0, 1), dtype=np.float32)
        log.info("rec stopped (%.2fs)", len(audio) / self.sr)
        buf = io.BytesIO()
        sf.write(buf, audio, self.sr, format="WAV", subtype="PCM_16")
        return buf.getvalue()

    def _on_audio(self, indata, frames, time_info, status) -> None:
        if status:
            log.debug("audio status: %s", status)
        if self._active:
            self._buf.append(indata.copy())


# ---------- asr ----------
class ASR:
    def __init__(self, cfg: Config):
        self.cfg = cfg

    def transcribe(self, wav: bytes) -> str:
        if not wav or len(wav) < 1024:
            return ""
        url = f"{self.cfg.base_url}/v1/audio/transcriptions"
        headers = {"Authorization": f"Bearer {self.cfg.api_key}"}
        files = {"file": ("speech.wav", wav, "audio/wav")}
        data = {"model": self.cfg.model, "language": self.cfg.lang}
        log.info("asr -> %s model=%s", url, self.cfg.model)
        r = requests.post(url, headers=headers, files=files, data=data, timeout=60)
        if r.status_code >= 400:
            raise RuntimeError(f"asr {r.status_code}: {r.text[:200]}")
        j = r.json()
        # accept openai-style or flat
        return (j.get("text") or j.get("data", {}).get("text") or "").strip()


# ---------- paste ----------
def paste(text: str) -> None:
    if not text:
        return
    import pyperclip
    import Quartz  # type: ignore

    prev = pyperclip.paste()
    pyperclip.copy(text)
    time.sleep(0.04)  # let clipboard settle
    # Cmd+V down/up
    down = Quartz.CGEventCreateKeyboardEvent(None, Quartz.kVK_ANSI_V, True)
    up = Quartz.CGEventCreateKeyboardEvent(None, Quartz.kVK_ANSI_V, False)
    down.setFlags(Quartz.kCGEventFlagMaskCommand)
    up.setFlags(Quartz.kCGEventFlagMaskCommand)
    Quartz.CGEventPost(Quartz.kCGHIDEventTap, down)
    Quartz.CGEventPost(Quartz.kCGHIDEventTap, up)
    # restore prior clipboard so we don't leak user's prior contents to nothing
    def _restore():
        time.sleep(0.3)
        try:
            pyperclip.copy(prev)
        except Exception:
            pass
    threading.Thread(target=_restore, daemon=True).start()


# ---------- hotkey ----------
def _parse_hotkey(spec: str):
    spec = spec.strip()
    if spec.startswith("Key."):
        return getattr(Key, spec[4:], None)
    if spec.startswith("char:"):
        return KeyCode.from_char(spec[5:])
    if len(spec) == 1:
        return KeyCode.from_char(spec)
    raise ValueError(f"bad hotkey: {spec}")


def run() -> None:
    logging.basicConfig(
        level=os.getenv("LOG_LEVEL", "INFO"),
        format="%(asctime)s %(levelname)s %(message)s",
        datefmt="%H:%M:%S",
    )
    cfg = Config.load()
    if not cfg.api_key:
        log.error("MINIMAX_API_KEY missing (set in .env)")
        sys.exit(1)

    target = _parse_hotkey(cfg.hotkey)
    if target is None:
        log.error("unknown hotkey: %s", cfg.hotkey)
        sys.exit(1)
    log.info("hotkey=%s model=%s base=%s", cfg.hotkey, cfg.model, cfg.base_url)
    print(f"Speek2Words ready. Hold {cfg.hotkey} to talk. Ctrl+C to quit.")

    rec = Recorder(cfg.sample_rate)
    asr = ASR(cfg)
    pressed_at: Optional[float] = None
    busy = threading.Lock()

    def on_press(key):
        nonlocal pressed_at
        if key != target or not busy.acquire(blocking=False):
            return
        try:
            rec.start()
            pressed_at = time.time()
        except Exception:
            busy.release()
            raise

    def on_release(key):
        nonlocal pressed_at
        if key != target or pressed_at is None:
            return
        try:
            wav = rec.stop()
            pressed_at = None
        except Exception as e:
            log.exception("rec stop: %s", e)
            busy.release()
            return
        try:
            text = asr.transcribe(wav)
        except Exception as e:
            log.error("asr failed: %s", e)
            busy.release()
            return
        if text:
            log.info("text: %s", text)
            try:
                paste(text)
            except Exception as e:
                log.error("paste failed: %s", e)
        busy.release()

    with keyboard.Listener(on_press=on_press, on_release=on_release, suppress=False) as listener:
        try:
            listener.join()
        except KeyboardInterrupt:
            print("\nbye")


if __name__ == "__main__":
    run()
