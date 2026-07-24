<script setup lang="ts">
import { ref, watch } from "vue";
import ItemsApi from "../api/ItemsApi";
import type { Item } from "../api/Item";

const props = defineProps<{
  item: Item;
}>();

const emit = defineEmits<{
  // Fired after `note.md` is successfully persisted, so the parent can refresh
  // the rendered Note tab and mark the asset as present.
  (e: "saved"): void;
}>();

const NOTE_FILENAME = "note.md";

const itemsApi = new ItemsApi();

const content = ref("");
const savedContent = ref("");
const loading = ref(false);
const saving = ref(false);
const error = ref<string | null>(null);

// Where attachments live for this item. Resolved asynchronously by
// useItems.ts; don't fall back to item.path or we'd fire a request against
// the wrong (non-existent) URL before it resolves.
const notePath = () => props.item.assetPath;

const load = async () => {
  // Skip the request entirely when /attachments already told us note.md
  // doesn't exist yet — a 404 would be expected but is a wasted round trip.
  if (!notePath() || !props.item.assets.includes(NOTE_FILENAME)) {
    content.value = "";
    savedContent.value = "";
    return;
  }

  loading.value = true;
  error.value = null;

  const response = await itemsApi.getAsset(notePath()!, NOTE_FILENAME);

  loading.value = false;

  if (response.ok) {
    content.value = response.body;
    savedContent.value = response.body;
    return;
  }

  // Absent note is the normal starting point, not an error.
  if (response.status === 404) {
    content.value = "";
    savedContent.value = "";
    return;
  }

  error.value = response.error;
};

// Persist on blur, but only when the text actually changed.
const save = async () => {
  if (saving.value || content.value === savedContent.value || !notePath()) return;

  saving.value = true;
  error.value = null;

  const file = new File([content.value], NOTE_FILENAME, { type: "text/markdown" });
  const response = await itemsApi.uploadAttachment(notePath()!, file);

  saving.value = false;

  if (response.ok) {
    savedContent.value = content.value;
    emit("saved");
    return;
  }

  error.value = response.error;
};

watch(
  () => props.item.assetPath ?? props.item.path,
  () => load(),
  { immediate: true },
);
</script>

<template>
  <div class="note-editor">
    <div class="note-head">
      <span class="note-title">Note</span>
      <span v-if="saving" class="note-state">Saving…</span>
      <span v-else-if="loading" class="note-state">Loading…</span>
    </div>

    <textarea
      v-model="content"
      class="note-input"
      placeholder="Write a note in Markdown…"
      spellcheck="false"
      :disabled="loading"
      @blur="save"
    />

    <p v-if="error" class="note-error">{{ error }}</p>
  </div>
</template>

<style scoped>
.note-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  box-sizing: border-box;
  padding: 1rem;
  gap: 0.625rem;
}
.note-head {
  display: flex;
  align-items: baseline;
  gap: 0.625rem;
}
.note-title {
  font-size: 0.8125rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text);
}
.note-state {
  font-size: 0.75rem;
  opacity: 0.7;
}
.note-input {
  flex: 1;
  min-height: 0;
  resize: none;
  font-family: var(--mono);
  font-size: 0.875rem;
  line-height: 150%;
  color: var(--text-h);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  padding: 0.75rem;
  box-sizing: border-box;
}
.note-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-bg);
}
.note-input:disabled {
  opacity: 0.6;
}
.note-error {
  font-size: 0.8125rem;
  color: var(--danger);
  margin: 0;
}
</style>
