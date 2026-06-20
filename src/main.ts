import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  register,
  unregister,
  isRegistered,
} from "@tauri-apps/plugin-global-shortcut";

// ---- State ----
let currentHotkey = "CmdOrCtrl+Shift+KeyK";
let isRecording = false;
let isChangingHotkey = false;

// ---- DOM helpers ----
function $(id: string): HTMLElement {
  return document.getElementById(id)!;
}
function $input(id: string): HTMLInputElement {
  return document.getElementById(id) as HTMLInputElement;
}
function $select(id: string): HTMLSelectElement {
  return document.getElementById(id) as HTMLSelectElement;
}

const statusEl = $("status");
const statusText = statusEl.querySelector(".status-text")!;
const recordingIndicator = $("recordingIndicator");
const apiKeyInput = $input("apiKey");
const toggleApiKeyBtn = $("toggleApiKey") as HTMLButtonElement;
const baseUrlInput = $input("baseUrl");
const modelInput = $input("model");
const hotkeyInput = $input("hotkey");
const changeHotkeyBtn = $("changeHotkey") as HTMLButtonElement;
const langSelect = $select("lang");
const saveBtn = $("saveBtn") as HTMLButtonElement;
const toastEl = $("toast");

// ---- Toast ----
let toastTimer: ReturnType<typeof setTimeout>;
function showToast(msg: string, type: "success" | "error" | "info" = "info") {
  clearTimeout(toastTimer);
  toastEl.textContent = msg;
  toastEl.className = `toast ${type} show`;
  toastTimer = setTimeout(() => {
    toastEl.classList.remove("show");
  }, 3000);
}

// ---- Config ----
async function loadConfig() {
  try {
    const pairs: [string, () => HTMLElement, string][] = [
      ["api_key", () => apiKeyInput, ""],
      ["base_url", () => baseUrlInput, "https://api.minimaxi.com"],
      ["model", () => modelInput, "speech-01"],
      ["hotkey", () => hotkeyInput, "CmdOrCtrl+Shift+KeyK"],
      ["lang", () => langSelect, "zh"],
    ];
    for (const [key, getEl, def] of pairs) {
      const val = await invoke<string | null>("get_config", { key });
      const v = val ?? def;
      const el = getEl();
      if (el instanceof HTMLSelectElement || el instanceof HTMLInputElement) {
        el.value = v;
      }
      if (key === "hotkey") {
        currentHotkey = v;
      }
    }
  } catch (e) {
    console.error("load config:", e);
  }
}

async function saveConfig() {
  try {
    const pairs: [string, string][] = [
      ["api_key", apiKeyInput.value.trim()],
      ["base_url", baseUrlInput.value.trim()],
      ["model", modelInput.value.trim()],
      ["hotkey", hotkeyInput.value],
      ["lang", langSelect.value],
    ];
    for (const [key, value] of pairs) {
      await invoke("set_config", { key, value });
    }
    await registerHotkey(hotkeyInput.value);
    showToast("Settings saved", "success");
  } catch (e) {
    showToast(`Save failed: ${e}`, "error");
  }
}

// ---- Hotkey ----
async function registerHotkey(shortcut: string) {
  try {
    if (await isRegistered(currentHotkey)) {
      await unregister(currentHotkey);
    }
    await register(shortcut, (event) => {
      if (event.state === "Pressed") {
        startRecording();
      } else {
        stopRecording();
      }
    });
    currentHotkey = shortcut;
  } catch (e) {
    showToast(`Hotkey register failed: ${e}`, "error");
  }
}

// ---- Recording ----
async function startRecording() {
  if (isRecording) return;
  isRecording = true;
  statusEl.classList.add("recording");
  statusText.textContent = "Recording...";
  recordingIndicator.classList.add("active");
  try {
    await invoke("start_recording");
  } catch (e) {
    showToast(`Record start failed: ${e}`, "error");
    resetRecordingState();
  }
}

async function stopRecording() {
  if (!isRecording) return;
  isRecording = false;
  statusText.textContent = "Transcribing...";
  recordingIndicator.classList.remove("active");
  try {
    const text = await invoke<string>("stop_and_transcribe");
    if (text) {
      statusText.textContent = "Typing...";
      await invoke("type_text", { text });
      showToast("Done", "success");
    } else {
      showToast("No speech detected", "info");
    }
  } catch (e) {
    showToast(`Transcribe failed: ${e}`, "error");
  }
  resetRecordingState();
}

function resetRecordingState() {
  isRecording = false;
  statusEl.classList.remove("recording");
  statusText.textContent = "Ready";
  recordingIndicator.classList.remove("active");
}

// ---- Tab Navigation ----
document.querySelectorAll<HTMLButtonElement>(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach((t) => t.classList.remove("active"));
    document.querySelectorAll(".page").forEach((p) => p.classList.remove("active"));
    tab.classList.add("active");
    const page = document.getElementById(`page-${tab.dataset.tab}`);
    page?.classList.add("active");
  });
});

// ---- API Key Toggle ----
toggleApiKeyBtn.addEventListener("click", () => {
  apiKeyInput.type = apiKeyInput.type === "password" ? "text" : "password";
});

// ---- Hotkey Change ----
changeHotkeyBtn.addEventListener("click", () => {
  if (isChangingHotkey) return;
  isChangingHotkey = true;
  hotkeyInput.value = "Press new hotkey...";
  hotkeyInput.style.color = "var(--accent)";

  const handler = (e: KeyboardEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const parts: string[] = [];
    if (e.metaKey || e.ctrlKey) parts.push("CmdOrCtrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");

    const key = e.key;
    if (
      key !== "Control" &&
      key !== "Shift" &&
      key !== "Alt" &&
      key !== "Meta"
    ) {
      let keyName = key.toUpperCase();
      if (keyName === " ") keyName = "Space";
      if (keyName.length === 1) keyName = `Key${keyName}`;
      parts.push(keyName);

      const shortcut = parts.join("+");
      hotkeyInput.value = shortcut;
      hotkeyInput.style.color = "";
      isChangingHotkey = false;
      document.removeEventListener("keydown", handler, true);
    }
  };

  document.addEventListener("keydown", handler, true);
});

// ---- Save ----
saveBtn.addEventListener("click", saveConfig);

// ---- Listen for backend events ----
listen("recording-started", () => {
  statusEl.classList.add("recording");
  statusText.textContent = "Recording...";
  recordingIndicator.classList.add("active");
});

listen("recording-stopped", () => {
  recordingIndicator.classList.remove("active");
  statusText.textContent = "Transcribing...";
});

listen("transcription-done", (e) => {
  const text = e.payload as string;
  if (text) {
    showToast(`"${text.substring(0, 30)}${text.length > 30 ? "..." : ""}"`, "success");
  }
  resetRecordingState();
});

listen("transcription-error", (e) => {
  showToast(`Error: ${e.payload}`, "error");
  resetRecordingState();
});

// ---- Init ----
async function init() {
  await loadConfig();
  await registerHotkey(currentHotkey);
}

init();
