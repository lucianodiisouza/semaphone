import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { applyTheme } from "./themes";
import { t, type Locale } from "./i18n";
import { playStageSound } from "./sounds";
import type { Config, Light } from "./types";

let currentLight: Light = "green";
let currentConfig: Config | null = null;

function setActiveLight(state: Light): void {
  document.querySelectorAll<HTMLElement>("[data-light]").forEach((el) => {
    const light = el.dataset.light as Light;
    el.classList.toggle("active", light === state);
  });
}

function applyMainLocale(locale: Locale): void {
  const strings = t(locale);
  const housing = document.querySelector(".housing") as HTMLElement | null;
  if (housing) {
    housing.title = strings.main.dragHint;
  }
  const settingsBtn = document.getElementById("settings-btn");
  if (settingsBtn) {
    settingsBtn.title = strings.main.settingsHint;
  }
}

async function loadConfig(): Promise<Config> {
  const config = await invoke<Config>("get_config");
  currentConfig = config;
  applyTheme(config.theme);
  applyMainLocale((config.locale as Locale) || "en");
  return config;
}

function handleStateChange(state: Light): void {
  if (state === currentLight) {
    return;
  }

  currentLight = state;
  setActiveLight(state);

  const sounds = currentConfig?.sounds;
  if (!sounds?.enabled) {
    return;
  }

  const stageSound = sounds[state];
  if (stageSound) {
    void playStageSound(state, stageSound);
  }
}

function setupDrag(): void {
  const housing = document.querySelector(".housing") as HTMLElement | null;
  housing?.addEventListener("mousedown", async (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    await getCurrentWindow().startDragging();
  });
}

window.addEventListener("DOMContentLoaded", async () => {
  await loadConfig();
  setActiveLight("green");
  setupDrag();

  await listen<{ state: Light }>("state-changed", (event) => {
    handleStateChange(event.payload.state);
  });

  await listen<Config>("config-changed", (event) => {
    currentConfig = event.payload;
    applyTheme(event.payload.theme);
    applyMainLocale(event.payload.locale as Locale);
  });

  document.getElementById("settings-btn")?.addEventListener("click", () => {
    invoke("show_settings");
  });

  const window = getCurrentWindow();
  window.onMoved(async () => {
    const pos = await window.outerPosition();
    const config = await invoke<Config>("get_config");
    config.window = { x: pos.x, y: pos.y };
    await invoke("save_config", { config });
  });
});
