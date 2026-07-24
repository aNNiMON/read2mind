<script setup lang="ts">
import useSettings, { ACCENTS, THEMES } from "../composables/useSettings";
import IconX from "~icons/lucide/x";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const { theme, accent } = useSettings();
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal" role="dialog" aria-modal="true" aria-label="Settings">
      <header class="modal-head">
        <h2 class="modal-title">Settings</h2>
        <button class="close-btn" type="button" title="Close" @click="emit('close')">
          <IconX />
        </button>
      </header>

      <section class="setting">
        <h3 class="setting-label">Theme</h3>
        <div class="theme-options" role="radiogroup" aria-label="Theme">
          <button
            v-for="option in THEMES"
            :key="option.value"
            type="button"
            class="theme-option"
            :class="{ active: theme === option.value }"
            role="radio"
            :aria-checked="theme === option.value"
            @click="theme = option.value"
          >
            {{ option.label }}
          </button>
        </div>
      </section>

      <section class="setting">
        <h3 class="setting-label">Accent color</h3>
        <div class="accent-options" role="radiogroup" aria-label="Accent color">
          <button
            v-for="option in ACCENTS"
            :key="option.value"
            type="button"
            class="accent-option"
            :class="{ active: accent === option.value }"
            role="radio"
            :aria-checked="accent === option.value"
            :title="option.value"
            :style="{ '--swatch': option.color }"
            @click="accent = option.value"
          >
            <span class="visually-hidden">{{ option.value }}</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 1.25rem;
}
.modal {
  width: 22rem;
  max-width: 100%;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  box-shadow: var(--shadow);
  padding: 1.5rem;
}
.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}
.modal-title {
  margin: 0;
  font-size: 1.25rem;
}
.close-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text);
  cursor: pointer;
}
.close-btn:hover {
  background: var(--accent-bg);
  color: var(--accent);
}
.setting {
  margin-bottom: 1.25rem;
}
.setting:last-child {
  margin-bottom: 0;
}
.setting-label {
  margin: 0 0 0.5rem;
  font-size: 0.8125rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text);
}
.theme-options {
  display: flex;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  overflow: hidden;
}
.theme-option {
  flex: 1;
  font: inherit;
  font-size: 0.875rem;
  padding: 0.4em 0;
  border: none;
  border-right: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-h);
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}
.theme-option:last-child {
  border-right: none;
}
.theme-option:hover {
  background: var(--accent-bg);
}
.theme-option.active {
  background: var(--accent-bg);
  color: var(--accent);
  font-weight: 600;
}
.accent-options {
  display: flex;
  gap: 0.625rem;
  flex-wrap: wrap;
}
.accent-option {
  width: 2rem;
  height: 2rem;
  border-radius: 50%;
  border: 2px solid transparent;
  background: var(--swatch);
  cursor: pointer;
  padding: 0;
  transition: transform 0.15s, box-shadow 0.15s;
}
.accent-option:hover {
  transform: scale(1.1);
}
.accent-option.active {
  border-color: var(--bg);
  box-shadow: 0 0 0 2px var(--text-h);
}
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
  white-space: nowrap;
}
</style>
