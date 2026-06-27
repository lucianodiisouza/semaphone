import { describe, expect, it } from "vitest";
import {
  DRAG_THRESHOLD_PX,
  pointerDistance,
  shouldOpenSettingsOnDoubleClick,
} from "./interaction";
import { applyWindowSize, WINDOW_SIZES } from "./window-size";
import { themeNames } from "./themes";
import { locales, type Locale } from "./i18n";

describe("interaction", () => {
  it("opens settings when movement is below threshold", () => {
    expect(shouldOpenSettingsOnDoubleClick(0)).toBe(true);
    expect(shouldOpenSettingsOnDoubleClick(DRAG_THRESHOLD_PX - 1)).toBe(true);
    expect(shouldOpenSettingsOnDoubleClick(DRAG_THRESHOLD_PX)).toBe(false);
    expect(shouldOpenSettingsOnDoubleClick(20)).toBe(false);
  });

  it("computes pointer distance", () => {
    expect(pointerDistance(0, 0, 3, 4)).toBe(5);
  });
});

describe("window-size", () => {
  it("applies valid size to body dataset", () => {
    document.body.dataset.size = "";
    expect(applyWindowSize("large")).toBe("large");
    expect(document.body.dataset.size).toBe("large");
  });

  it("falls back to medium for unknown size", () => {
    expect(applyWindowSize("huge")).toBe("medium");
    expect(document.body.dataset.size).toBe("medium");
  });

  it("exports three size presets", () => {
    expect(WINDOW_SIZES).toEqual(["small", "medium", "large"]);
  });
});

describe("themes", () => {
  it("includes builtin themes", () => {
    expect(themeNames).toContain("classic");
    expect(themeNames).toContain("minimal");
    expect(themeNames).toContain("neon");
  });
});

describe("i18n streamdeck onboarding", () => {
  it.each<Locale>(["en", "pt-BR"])("includes streamdeck strings for %s", (locale) => {
    const onboarding = locales[locale].onboarding;
    expect(onboarding.streamdeckTitle).toBeTruthy();
    expect(onboarding.streamdeckBody).toBeTruthy();
    expect(onboarding.streamdeckLabel).toBeTruthy();
    expect(onboarding.streamdeckNote).toBeTruthy();
    expect(onboarding.streamdeckInstalling).toBeTruthy();
    expect(onboarding.streamdeckDone).toBeTruthy();
  });
});
