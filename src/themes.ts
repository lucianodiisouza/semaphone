import classic from "./themes/classic.json";
import minimal from "./themes/minimal.json";
import neon from "./themes/neon.json";

export interface ThemeTokens {
  housingBg: string;
  housingBorder: string;
  housingShadow: string;
  lensOff: string;
  green: string;
  greenGlow: string;
  yellow: string;
  yellowGlow: string;
  red: string;
  redGlow: string;
}

interface ThemeFile extends ThemeTokens {
  themeVersion: number;
  id: string;
  name: string;
}

const builtin: Record<string, ThemeFile> = {
  classic: classic as ThemeFile,
  minimal: minimal as ThemeFile,
  neon: neon as ThemeFile,
};

export const themeNames = Object.keys(builtin);

export function applyTheme(name: string): void {
  const theme = builtin[name] ?? builtin.classic;
  const root = document.documentElement;
  root.style.setProperty("--housing-bg", theme.housingBg);
  root.style.setProperty("--housing-border", theme.housingBorder);
  root.style.setProperty("--housing-shadow", theme.housingShadow);
  root.style.setProperty("--lens-off", theme.lensOff);
  root.style.setProperty("--green", theme.green);
  root.style.setProperty("--green-glow", theme.greenGlow);
  root.style.setProperty("--yellow", theme.yellow);
  root.style.setProperty("--yellow-glow", theme.yellowGlow);
  root.style.setProperty("--red", theme.red);
  root.style.setProperty("--red-glow", theme.redGlow);
  document.body.dataset.theme = theme.id;
}
