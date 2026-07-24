<script setup lang="ts">
import type { ContentTab } from "../api/contentTypes";
import IconBookOpen from "~icons/lucide/book-open";
import IconPencil from "~icons/lucide/pencil";

const props = defineProps<{
  tabs: ContentTab[];
  assets: string[];
  active: ContentTab;
  editing: boolean;
}>();

const emit = defineEmits<{
  (e: "select", tab: ContentTab): void;
  (e: "toggle-edit", tab: ContentTab): void;
}>();

const isMissing = (tab: ContentTab) => !props.assets.includes(tab.filename);
</script>

<template>
  <div class="content-tabs" role="tablist">
    <span v-for="tab in tabs" :key="tab.key" class="tab-wrap">
      <button
        class="tab"
        type="button"
        role="tab"
        :class="{ active: tab.key === active.key, missing: isMissing(tab) }"
        :aria-selected="tab.key === active.key"
        :title="isMissing(tab) ? `${tab.label} — no content yet` : tab.label"
        @click="emit('select', tab)"
      >
        {{ tab.label }}
      </button>

      <span
        v-if="tab.editable && tab.key === active.key && !isMissing(tab)"
        class="edit-toggle"
        role="button"
        tabindex="0"
        :title="editing ? 'Read Markdown' : 'Edit Markdown'"
        :aria-label="editing ? 'Read Markdown' : 'Edit Markdown'"
        @click.stop="emit('toggle-edit', tab)"
        @keydown.enter.stop="emit('toggle-edit', tab)"
        @keydown.space.stop.prevent="emit('toggle-edit', tab)"
      >
        <IconBookOpen v-if="editing" />
        <IconPencil v-else />
      </span>
    </span>
  </div>
</template>

<style scoped>
.content-tabs {
  display: inline-flex;
  align-self: flex-start;
  max-width: 100%;
  margin-bottom: 1rem;
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
}
.tab-wrap {
  display: inline-flex;
  align-items: stretch;
}
.tab-wrap:last-child .tab:not(:has(+ .edit-toggle)) {
  border-right: none;
}
.tab {
  font: inherit;
  font-size: 0.9em;
  padding: 0.3em 0.8em;
  white-space: nowrap;
  border: none;
  border-right: 1px solid var(--border);
  background: var(--bg);
  color: var(--text-h);
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}
.tab:hover {
  background: var(--accent-bg);
}
.tab.active {
  background: var(--accent-bg);
  color: var(--accent);
}
.edit-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 0.625rem;
  font-size: 0.875rem;
  border-right: 1px solid var(--border);
  background: var(--accent-bg);
  color: var(--accent);
  cursor: pointer;
}
.tab-wrap:last-child .edit-toggle {
  border-right: none;
}
.edit-toggle:hover {
  opacity: 0.8;
}
/* Asset not uploaded yet: dimmed and dashed, but still clickable. */
.tab.missing {
  color: var(--text);
  opacity: 0.55;
  font-style: italic;
  background: repeating-linear-gradient(
    45deg,
    transparent,
    transparent 5px,
    var(--code-bg) 5px,
    var(--code-bg) 6px
  );
}
.tab.missing.active {
  opacity: 0.85;
  color: var(--accent);
}
</style>
