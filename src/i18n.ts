export type Locale = "en" | "pt-BR";

export interface LocaleStrings {
  settings: {
    title: string;
    theme: string;
    language: string;
    stealth: string;
    connect: string;
    cancel: string;
    save: string;
    stealthNote: string;
  };
  tools: {
    cursor: string;
    claude: string;
    codex: string;
    gemini: string;
    copilot: string;
    all: string;
    connected: string;
    failed: string;
  };
}

export const locales: Record<Locale, LocaleStrings> = {
  en: {
    settings: {
      title: "Settings",
      theme: "Theme",
      language: "Language",
      stealth: "Stealth mode (hide from screen share)",
      connect: "Connect tools",
      cancel: "Cancel",
      save: "Save",
      stealthNote:
        "Stealth works best on Windows. On macOS 15+ some capture tools may still record the window.",
    },
    tools: {
      cursor: "Cursor",
      claude: "Claude Code",
      codex: "Codex CLI",
      gemini: "Gemini CLI",
      copilot: "Copilot CLI",
      all: "Connect all",
      connected: "Hooks installed",
      failed: "Install failed",
    },
  },
  "pt-BR": {
    settings: {
      title: "Configurações",
      theme: "Tema",
      language: "Idioma",
      stealth: "Modo stealth (ocultar no compartilhamento de tela)",
      connect: "Conectar ferramentas",
      cancel: "Cancelar",
      save: "Salvar",
      stealthNote:
        "Stealth funciona melhor no Windows. No macOS 15+ algumas ferramentas ainda podem capturar a janela.",
    },
    tools: {
      cursor: "Cursor",
      claude: "Claude Code",
      codex: "Codex CLI",
      gemini: "Gemini CLI",
      copilot: "Copilot CLI",
      all: "Conectar todas",
      connected: "Hooks instalados",
      failed: "Falha na instalação",
    },
  },
};

export function t(locale: Locale): LocaleStrings {
  return locales[locale] ?? locales.en;
}
