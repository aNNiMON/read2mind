import { ref, watchEffect } from "vue";

export type Theme = "light" | "dark" | "system";
export type Accent = "purple" | "green" | "yellow" | "orange" | "blue" | "gray";

export const THEMES: { value: Theme; label: string }[] = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
];

// Swatch colors for the settings UI; the actual theme-aware values live in
// style.css under the matching [data-accent] selectors.
export const ACCENTS: { value: Accent; color: string }[] = [
  { value: "purple", color: "#a855f7" },
  { value: "green", color: "#10b981" },
  { value: "yellow", color: "#eab308" },
  { value: "orange", color: "#f97316" },
  { value: "blue", color: "#3b82f6" },
  { value: "gray", color: "#71717a" },
];

const THEME_KEY = "settings.theme";
const ACCENT_KEY = "settings.accent";

const isTheme = (value: string | null): value is Theme =>
  THEMES.some((option) => option.value === value);

const isAccent = (value: string | null): value is Accent =>
  ACCENTS.some((option) => option.value === value);

const stored = (key: string) => {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
};

const storedTheme = stored(THEME_KEY);
const storedAccent = stored(ACCENT_KEY);

const theme = ref<Theme>(isTheme(storedTheme) ? storedTheme : "system");
const accent = ref<Accent>(isAccent(storedAccent) ? storedAccent : "purple");

// Reflect the settings on <html>; style.css keys all theme/accent variables
// off these attributes ('system' delegates to prefers-color-scheme).
watchEffect(() => {
  const root = document.documentElement;
  root.dataset.theme = theme.value;
  root.dataset.accent = accent.value;
  try {
    localStorage.setItem(THEME_KEY, theme.value);
    localStorage.setItem(ACCENT_KEY, accent.value);
  } catch {
    // Private mode etc. — settings just won't persist.
  }
});

export default function useSettings() {
  return { theme, accent };
}
