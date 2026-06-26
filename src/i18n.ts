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
    sounds: string;
    soundsEnabled: string;
    soundsNote: string;
    soundGreen: string;
    soundYellow: string;
    soundRed: string;
    soundPreset: string;
    soundCustom: string;
    soundBrowse: string;
    soundPreview: string;
    soundCustomActive: string;
    soundImportFailed: string;
    soundTooLarge: string;
    soundTooLong: string;
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
  main: {
    dragHint: string;
    settingsHint: string;
  };
  about: {
    title: string;
    description: string;
    lights: string[];
    controlsTitle: string;
    controls: string[];
    trayTitle: string;
    trayMenu: string[];
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
      sounds: "Notification sounds",
      soundsEnabled: "Play a sound when the light changes",
      soundsNote: "Custom files must be under 512 KB and 3 seconds. Built-in sounds are short tones.",
      soundGreen: "Green (ready)",
      soundYellow: "Yellow (thinking)",
      soundRed: "Red (writing)",
      soundPreset: "Sound",
      soundCustom: "Custom file…",
      soundBrowse: "Browse…",
      soundPreview: "Preview",
      soundCustomActive: "Using custom file",
      soundImportFailed: "Could not import audio file. Check format and size.",
      soundTooLarge: "File is too large (max 512 KB).",
      soundTooLong: "Audio is too long (max 3 seconds).",
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
    main: {
      dragHint: "Click and drag here to move",
      settingsHint: "Settings",
    },
    about: {
      title: "About Semaphore",
      description:
        "Floating traffic light for AI coding agents. See at a glance when your agent is idle, thinking, or writing files.",
      lights: [
        "Green — ready for a new task",
        "Yellow — thinking / running tools",
        "Red — writing or editing files",
      ],
      controlsTitle: "Controls",
      controls: [
        "Drag the traffic light body to move the widget",
        "Hover the widget and click ⚙ to open Settings",
        "Left-click the tray icon to show the widget",
      ],
      trayTitle: "Tray menu (right-click tray icon)",
      trayMenu: [
        "Show Semaphore — show the floating widget",
        "Hide Window — hide the widget",
        "Settings — open this window",
        "Toggle Stealth — hide from screen capture",
        "Always on Top — keep the widget above other windows",
        "Quit — exit Semaphore",
      ],
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
      sounds: "Sons de notificação",
      soundsEnabled: "Tocar um som quando a luz mudar",
      soundsNote: "Arquivos personalizados devem ter até 512 KB e 3 segundos. Sons integrados são tons curtos.",
      soundGreen: "Verde (pronto)",
      soundYellow: "Amarelo (pensando)",
      soundRed: "Vermelho (escrevendo)",
      soundPreset: "Som",
      soundCustom: "Arquivo personalizado…",
      soundBrowse: "Procurar…",
      soundPreview: "Ouvir",
      soundCustomActive: "Usando arquivo personalizado",
      soundImportFailed: "Não foi possível importar o áudio. Verifique formato e tamanho.",
      soundTooLarge: "Arquivo muito grande (máx. 512 KB).",
      soundTooLong: "Áudio muito longo (máx. 3 segundos).",
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
    main: {
      dragHint: "Clique e arraste aqui para mover",
      settingsHint: "Configurações",
    },
    about: {
      title: "Sobre o Semaphore",
      description:
        "Semáforo flutuante para agentes de IA. Veja de relance quando seu agente está ocioso, pensando ou editando arquivos.",
      lights: [
        "Verde — pronto para uma nova tarefa",
        "Amarelo — pensando / executando ferramentas",
        "Vermelho — escrevendo ou editando arquivos",
      ],
      controlsTitle: "Controles",
      controls: [
        "Arraste o corpo do semáforo para mover o widget",
        "Passe o mouse e clique em ⚙ para abrir Configurações",
        "Clique esquerdo no ícone da bandeja para mostrar o widget",
      ],
      trayTitle: "Menu da bandeja (clique direito no ícone)",
      trayMenu: [
        "Show Semaphore — mostra o widget flutuante",
        "Hide Window — oculta o widget",
        "Settings — abre esta janela",
        "Toggle Stealth — oculta da captura de tela",
        "Always on Top — mantém o widget acima das outras janelas",
        "Quit — encerra o Semaphore",
      ],
    },
  },
};

export function t(locale: Locale): LocaleStrings {
  return locales[locale] ?? locales.en;
}
