import { getCurrentWindow, Effect, EffectState } from "@tauri-apps/api/window";

export type ThemeMode = "light" | "dark" | "auto";

let currentMode: ThemeMode = "auto";

function applyTheme(theme: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", theme);
}

async function detectSystemTheme(): Promise<"light" | "dark"> {
  try {
    const win = getCurrentWindow();
    const theme = await win.theme();
    return theme === "dark" ? "dark" : "light";
  } catch {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
}

export async function initTheme() {
  // Apply Mica effect
  try {
    const win = getCurrentWindow();
    await win.setEffects({
      effects: [Effect.Mica],
      state: EffectState.FollowsWindowActiveState,
    });
  } catch (e) {
    // Mica not available (Windows 10 or dev mode) — try Acrylic fallback
    try {
      const win = getCurrentWindow();
      await win.setEffects({
        effects: [Effect.Acrylic],
        state: EffectState.FollowsWindowActiveState,
      });
    } catch {
      // No blur effects available, solid background will be used
    }
  }

  // Apply initial theme
  const resolved = await detectSystemTheme();
  applyTheme(resolved);

  // Listen for system theme changes (for auto mode)
  try {
    const win = getCurrentWindow();
    await win.onThemeChanged(({ payload }) => {
      if (currentMode === "auto") {
        applyTheme(payload === "dark" ? "dark" : "light");
      }
    });
  } catch {
    // Fallback: use matchMedia listener
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (e) => {
        if (currentMode === "auto") {
          applyTheme(e.matches ? "dark" : "light");
        }
      });
  }
}

export async function setThemeMode(mode: ThemeMode) {
  currentMode = mode;
  if (mode === "auto") {
    const resolved = await detectSystemTheme();
    applyTheme(resolved);
  } else {
    applyTheme(mode);
  }
}

export function getThemeMode(): ThemeMode {
  return currentMode;
}
