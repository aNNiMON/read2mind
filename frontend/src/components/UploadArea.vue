<script setup lang="ts">
import { ref } from "vue";
import ItemsApi from "../api/ItemsApi";
import type { ContentTab } from "../api/contentTypes";
import IconUpload from "~icons/lucide/upload";

const props = defineProps<{
  path: string;
  tab: ContentTab;
}>();

const emit = defineEmits<{
  (e: "uploaded"): void;
}>();

const itemsApi = new ItemsApi();

const dragging = ref(false);
const uploading = ref(false);
const error = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);

const upload = async (file: File) => {
  uploading.value = true;
  error.value = null;

  const response = await itemsApi.uploadAttachment(props.path, file);

  uploading.value = false;

  if (response.ok) {
    emit("uploaded");
  } else {
    error.value = response.error;
  }
};

const onPick = (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) upload(file);
};

const onDrop = (event: DragEvent) => {
  dragging.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file) upload(file);
};
</script>

<template>
  <div
    class="upload-area"
    :class="{ dragging }"
    @dragover.prevent="dragging = true"
    @dragleave.prevent="dragging = false"
    @drop.prevent="onDrop"
  >
    <h2 class="no-content">No Content</h2>
    <p class="hint">
      Drag &amp; drop a file here to set <code>{{ tab.filename }}</code>, or
    </p>

    <button class="upload-btn" type="button" :disabled="uploading" @click="fileInput?.click()">
      <IconUpload />
      {{ uploading ? "Uploading…" : "Upload" }}
    </button>

    <input ref="fileInput" type="file" hidden @change="onPick" />

    <p v-if="error" class="upload-error">{{ error }}</p>
  </div>
</template>

<style scoped>
.upload-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  flex: 1;
  min-height: 18rem;
  margin: 1.5rem 0;
  border: 2px dashed var(--border);
  border-radius: 0.75rem;
  padding: 2rem;
  text-align: center;
  transition: border-color 0.2s, background 0.2s;
}
.upload-area.dragging {
  border-color: var(--accent);
  background: var(--accent-bg);
}
.no-content {
  font-size: 2rem;
  margin: 0;
}
.hint {
  font-size: 0.875rem;
  color: var(--text);
}
.upload-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  font: inherit;
  font-size: 0.9375rem;
  padding: 0.5em 1.2em;
  border: none;
  border-radius: 0.375rem;
  background: var(--accent);
  color: var(--bg);
  cursor: pointer;
}
.upload-btn:disabled {
  opacity: 0.6;
  cursor: default;
}
.upload-error {
  font-size: 0.875rem;
  color: var(--danger);
}
</style>
