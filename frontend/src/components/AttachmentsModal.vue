<script setup lang="ts">
import { computed, ref } from "vue";
import type { DeleteAttachmentResp, Item, UploadResp } from "../api/Item";
import ItemsApi from "../api/ItemsApi";
import { formatFileSize } from "../utils/format";
import IconCopy from "~icons/lucide/copy";
import IconTrash from "~icons/lucide/trash-2";
import IconUpload from "~icons/lucide/upload";
import IconX from "~icons/lucide/x";

const itemsApi = new ItemsApi();

const props = defineProps<{
  item: Item;
  deleteAttachment: (path: string, filename: string) => Promise<DeleteAttachmentResp>;
  uploadAttachment: (path: string, file: File) => Promise<UploadResp>;
}>();

const emit = defineEmits<{ close: [] }>();

const attachmentError = ref<string | null>(null);
const deletingAttachment = ref<string | null>(null);

// The assets list also carries generated files; hide `content.md` here.
const attachments = computed(() => props.item.assets.filter((asset) => asset !== "content.md"));

const attachmentUrl = (filename: string) => {
  const assetPath = props.item.assetPath ?? props.item.path;
  return itemsApi.assetUrl(assetPath, filename);
};

const copiedAttachment = ref<string | null>(null);

const asAttachmentLink = (filename: string) => {
  const mdLink = `[${filename}](${filename})`;
  if (filename.match(/\.(png|jpg|jpeg|gif|webp|avif)$/i)) {
    return `!${mdLink}`;
  }
  return mdLink;
};

const copyAttachmentUrl = async (filename: string) => {
  await navigator.clipboard.writeText(asAttachmentLink(filename));
  copiedAttachment.value = filename;
  setTimeout(() => {
    if (copiedAttachment.value === filename) copiedAttachment.value = null;
  }, 1500);
};

const removeAttachment = async (filename: string) => {
  if (deletingAttachment.value) return;

  const confirmed = window.confirm(`Delete "${filename}"? This cannot be undone.`);
  if (!confirmed) return;

  deletingAttachment.value = filename;
  attachmentError.value = null;

  const response = await props.deleteAttachment(props.item.path, filename);

  deletingAttachment.value = null;

  if (!response.ok) {
    attachmentError.value = response.error;
  }
};

const dragging = ref(false);
const uploading = ref(false);
const uploadError = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);

const uploadFile = async (file: File) => {
  uploading.value = true;
  uploadError.value = null;

  const response = await props.uploadAttachment(props.item.path, file);

  uploading.value = false;
  if (response.ok) {
    props.item.attachmentsMetadata[file.name] = {
      name: file.name,
      size: file.size,
    };
  } else {
    uploadError.value = response.error;
  }
};

const onPick = (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) uploadFile(file);
  (event.target as HTMLInputElement).value = "";
};

const onDrop = (event: DragEvent) => {
  dragging.value = false;
  const file = event.dataTransfer?.files?.[0];
  if (file) uploadFile(file);
};
</script>

<template>
  <div class="attach-backdrop" @click.self="emit('close')">
    <div class="attach-modal" role="dialog" aria-modal="true" aria-label="Attachments">
      <header class="attach-head">
        <h2 class="attach-title">Attachments</h2>
        <button class="attach-close" type="button" title="Close" @click="emit('close')">
          <IconX />
        </button>
      </header>

      <div
        class="attach-upload"
        :class="{ dragging, uploading }"
        @dragover.prevent="dragging = true"
        @dragleave.prevent="dragging = false"
        @drop.prevent="onDrop"
        @click="fileInput?.click()"
      >
        <IconUpload />
        <span>{{ uploading ? "Uploading…" : "Drag & drop a file here, or click to choose" }}</span>
        <input ref="fileInput" type="file" hidden @change="onPick" />
      </div>
      <p v-if="uploadError" class="attach-error">{{ uploadError }}</p>

      <p v-if="attachmentError" class="attach-error">{{ attachmentError }}</p>

      <ul v-if="attachments.length" class="attach-list">
        <li v-for="filename in attachments" :key="filename" class="attach-row">
          <a
            class="attach-name"
            :href="attachmentUrl(filename)"
            target="_blank"
            rel="noopener noreferrer"
          >{{ filename }}</a>
          <span class="attach-size">{{ formatFileSize(props.item.attachmentsMetadata[filename]?.size ?? 0) }}</span>
          <button
            type="button"
            class="attach-copy"
            aria-label="Copy attachment path"
            :title="copiedAttachment === filename ? 'Copied!' : 'Copy attachment path'"
            @click="copyAttachmentUrl(filename)"
          >
            <IconCopy />
          </button>
          <button
            type="button"
            class="attach-del"
            :disabled="deletingAttachment === filename"
            aria-label="Delete attachment"
            title="Delete attachment"
            @click="removeAttachment(filename)"
          >
            <IconTrash />
          </button>
        </li>
      </ul>
      <p v-else class="attach-empty">No attachments.</p>
    </div>
  </div>
</template>

<style scoped>
.attach-backdrop {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: color-mix(in srgb, #000 45%, transparent);
}
.attach-modal {
  display: flex;
  flex-direction: column;
  width: min(28rem, 100%);
  max-height: min(70vh, 34rem);
  box-sizing: border-box;
  padding: 1.25rem;
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  background: var(--bg);
  box-shadow: var(--shadow);
}
.attach-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}
.attach-title {
  font-size: 1.1rem;
  margin: 0;
}
.attach-close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.875rem;
  height: 1.875rem;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text);
  cursor: pointer;
}
.attach-close:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}
.attach-upload {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  padding: 1rem;
  margin-bottom: 0.75rem;
  border: 2px dashed var(--border);
  border-radius: 0.5rem;
  font-size: 0.8125rem;
  color: var(--text);
  text-align: center;
  cursor: pointer;
  transition: border-color 0.2s, background 0.2s;
}
.attach-upload:hover {
  border-color: var(--accent-border);
}
.attach-upload.dragging {
  border-color: var(--accent);
  background: var(--accent-bg);
}
.attach-upload.uploading {
  cursor: default;
  opacity: 0.7;
}
.attach-error {
  font-size: 0.8125rem;
  color: var(--danger);
  margin: 0 0 0.5rem;
}
.attach-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  min-height: 0;
}
.attach-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.25rem;
  border-bottom: 1px solid var(--border);
}
.attach-name {
  flex: 1;
  min-width: 0;
  font-size: 0.875rem;
  color: var(--text-h);
  overflow-wrap: anywhere;
  text-decoration: none;
}
.attach-name:hover {
  color: var(--accent);
  text-decoration: underline;
}
.attach-size {
  font-size: 0.75rem;
  color: var(--text);
  opacity: 0.7;
}
.attach-copy {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text);
  cursor: pointer;
}
.attach-copy:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}
.attach-del {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--danger);
  cursor: pointer;
}
.attach-del:hover {
  background: color-mix(in srgb, var(--danger) 12%, transparent);
}
.attach-del:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.attach-empty {
  font-size: 0.875rem;
  color: var(--text);
  opacity: 0.7;
  margin: 0.5rem 0 0;
}
</style>
