import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { applyTheme } from "./themes";
import { t, type Locale } from "./i18n";

type Light = "green" | "yellow" | "red";

interface Config {
  idle_timeout_secs: number;
  stealth: boolean;
  stealth_acknowledged: boolean;
  theme: string;
  locale: string;
  window: { x: number; y: number };
}

let currentLocale: Locale = "en";

function setActiveLight(state: Light): void {
  document.querySelectorAll<HTMLElement>("[data-light]").forEach((el) => {
    const light = el.dataset.light as Light;
    el.classList.toggle("active", light === state);
  });
}

function applyLocale(locale: Locale): void {
  currentLocale = locale;
  const strings = t(locale);
  document.getElementById("settings-title")!.textContent = strings.settings.title;
  document.getElementById("label-theme")!.textContent = strings.settings.theme;
  document.getElementById("label-language")!.textContent = strings.settings.language;
  document.getElementById("label-stealth")!.textContent = strings.settings.stealth;
  document.getElementById("label-connect")!.textContent = strings.settings.connect;
  document.getElementById("btn-cancel")!.textContent = strings.settings.cancel;
  document.getElementById("btn-save")!.textContent = strings.settings.save;
  document.getElementById("stealth-note")!.textContent = strings.settings.stealthNote;
  document.getElementById("connect-cursor")!.textContent = strings.tools.cursor;
  document.getElementById("connect-claude")!.textContent = strings.tools.claude;
  document.getElementById("connect-codex")!.textContent = strings.tools.codex;
  document.getElementById("connect-gemini")!.textContent = strings.tools.gemini;
  document.getElementById("connect-copilot")!.textContent = strings.tools.copilot;
  document.getElementById("connect-all")!.textContent = strings.tools.all;
}

async function loadConfig(): Promise<Config> {
  const config = await invoke<Config>("get_config");
  applyTheme(config.theme);
  applyLocale((config.locale as Locale) || "en");
  (document.getElementById("theme-select") as HTMLSelectElement).value = config.theme;
  (document.getElementById("locale-select") as HTMLSelectElement).value = config.locale;
  (document.getElementById("stealth-checkbox") as HTMLInputElement).checked = config.stealth;
  return config;
}

async function maybeAcknowledgeStealth(config: Config): Promise<Config> {
  const checkbox = document.getElementById("stealth-checkbox") as HTMLInputElement;
  if (!checkbox.checked || config.stealth_acknowledged) {
    return config;
  }
  const strings = t(currentLocale);
  const ok = confirm(strings.settings.stealthNote);
  if (!ok) {
    checkbox.checked = false;
    config.stealth = false;
    return config;
  }
  config.stealth_acknowledged = true;
  return config;
}

async function saveConfigFromForm(): Promise<void> {
  let config = await invoke<Config>("get_config");
  config.theme = (document.getElementById("theme-select") as HTMLSelectElement).value;
  config.locale = (document.getElementById("locale-select") as HTMLSelectElement).value;
  config.stealth = (document.getElementById("stealth-checkbox") as HTMLInputElement).checked;
  config = await maybeAcknowledgeStealth(config);
  await invoke("save_config", { config });
  applyTheme(config.theme);
  applyLocale(config.locale as Locale);
  await invoke("set_stealth", { enabled: config.stealth });
}

async function connectTool(tool: string): Promise<void> {
  const strings = t(currentLocale);
  try {
    await invoke("install_hooks", { tool });
    alert(strings.tools.connected);
  } catch {
    alert(strings.tools.failed);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  await loadConfig();
  setActiveLight("green");

  await listen<{ state: Light }>("state-changed", (event) => {
    setActiveLight(event.payload.state);
  });

  const dialog = document.getElementById("settings-dialog") as HTMLDialogElement;
  document.getElementById("settings-btn")?.addEventListener("click", () => {
    dialog.showModal();
  });

  document.getElementById("settings-form")?.addEventListener("close", async (e) => {
    if ((e.target as HTMLDialogElement).returnValue === "default") {
      await saveConfigFromForm();
    }
  });

  document.getElementById("connect-cursor")?.addEventListener("click", () => connectTool("cursor"));
  document.getElementById("connect-claude")?.addEventListener("click", () => connectTool("claude-code"));
  document.getElementById("connect-codex")?.addEventListener("click", () => connectTool("codex"));
  document.getElementById("connect-gemini")?.addEventListener("click", () => connectTool("gemini-cli"));
  document.getElementById("connect-copilot")?.addEventListener("click", () => connectTool("copilot-cli"));
  document.getElementById("connect-all")?.addEventListener("click", () => connectTool("all"));

  const window = getCurrentWindow();
  window.onMoved(async () => {
    const pos = await window.outerPosition();
    const config = await invoke<Config>("get_config");
    config.window = { x: pos.x, y: pos.y };
    await invoke("save_config", { config });
  });
});
