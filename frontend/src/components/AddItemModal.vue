<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import type { CreateItemPayload, CreateResp, ItemKind } from "../api/Item";
import { parseCreatedAtInput } from "../utils/date";

type ItemPayload = CreateItemPayload;

const props = defineProps<{
  kind: ItemKind;
  allTags: string[];
  submit: (kind: ItemKind, payload: ItemPayload) => Promise<CreateResp>;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const KIND_LABELS: Record<ItemKind, string> = {
  article: "article",
  note: "note",
  task: "task",
  video: "video",
};

const isLink = computed(() => props.kind === "article" || props.kind === "video");
const isText = computed(() => props.kind === "note" || props.kind === "task");

const url = ref("");
const title = ref("");
const tags = ref<string[]>([]);
const createdAt = ref("");
const html = ref("");
const transcript = ref("");
const content = ref("");

const submitting = ref(false);
const error = ref<string | null>(null);

const titleInput = ref<HTMLInputElement | null>(null);
const urlInput = ref<HTMLInputElement | null>(null);

const onKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") emit("close");
};

onMounted(() => {
  nextTick(() => {
    (isText.value ? titleInput.value : urlInput.value)?.focus();
  });
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});

const submit = async () => {
  if (isLink.value && !url.value.trim()) {
    error.value = "URL is required.";
    return;
  }
  if (isText.value && !content.value.trim()) {
    error.value = "Content is required.";
    return;
  }

  const createdAtInput = createdAt.value.trim();
  let createdAtApiValue: string | undefined;
  if (createdAtInput) {
    createdAtApiValue = parseCreatedAtInput(createdAtInput) ?? undefined;
    if (!createdAtApiValue) {
      error.value = "Date must be YYYY-MM-DD or YYYY-MM-DD HH:mm:ss.";
      return;
    }
  }

  submitting.value = true;
  error.value = null;

  const payload: ItemPayload = { kind: props.kind };
  if (isLink.value) payload.url = url.value.trim();
  if (title.value.trim()) payload.title = title.value.trim();
  if (tags.value.length) payload.tags = tags.value.join(",");
  if (createdAtApiValue) payload.created_at = createdAtApiValue;
  if (props.kind === "article" && html.value.trim()) payload.html = html.value;
  if (props.kind === "video" && transcript.value.trim()) payload.transcript = transcript.value;
  if (isText.value) payload.content = content.value;

  const result = await props.submit(props.kind, payload);

  submitting.value = false;

  if (result?.ok) {
    emit("close");
  } else {
    error.value = result?.error ?? `Failed to create ${KIND_LABELS[props.kind]}.`;
  }
};
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal" role="dialog" aria-modal="true" :aria-label="`Add ${KIND_LABELS[kind]}`">
      <h2 class="modal-title">Add {{ KIND_LABELS[kind] }}</h2>

      <form class="form" @submit.prevent="submit">
        <label v-if="isLink">
          <span>URL <em>*</em></span>
          <input ref="urlInput" v-model="url" type="url" required
            placeholder="https://example.com" />
        </label>

        <label v-if="isText">
          <span>Title</span>
          <input ref="titleInput" v-model="title" type="text" placeholder="" />
        </label>

        <label v-if="isText">
          <span>Content <em>*</em></span>
          <textarea v-model="content" rows="6" placeholder="Markdown content"></textarea>
        </label>

        <label v-if="kind === 'article'">
          <span>HTML</span>
          <textarea v-model="html" rows="5" placeholder="Raw HTML content"></textarea>
        </label>

        <label v-if="kind === 'video'">
          <span>Transcript</span>
          <textarea v-model="transcript" rows="5"
            placeholder="Transcript (Markdown)"></textarea>
        </label>

        <p v-if="error" class="form-error">{{ error }}</p>

        <div class="form-actions">
          <button type="button" class="btn ghost" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn primary" :disabled="submitting">
            {{ submitting ? "Saving…" : "Save" }}
          </button>
        </div>
      </form>
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
  width: 32.5rem;
  max-width: 100%;
  max-height: 90dvh;
  overflow-y: auto;
  box-sizing: border-box;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  box-shadow: var(--shadow);
  padding: 1.5rem;
}
.modal-title {
  margin: 0 0 1rem;
  text-transform: capitalize;
}
.form {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.875rem;
  color: var(--text);
}
.form label em {
  color: var(--accent);
  font-style: normal;
}
.form input,
.form textarea {
  font: inherit;
  font-size: 0.9375rem;
  padding: 0.5em 0.65em;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text-h);
  resize: vertical;
}
.form input:focus,
.form textarea:focus {
  outline: none;
  border-color: var(--accent);
}
.form-error {
  color: var(--danger);
  font-size: 0.875rem;
}
.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.625rem;
  margin-top: 0.25rem;
}
.btn {
  font: inherit;
  font-size: 0.9375rem;
  padding: 0.5em 1.2em;
  border-radius: 0.375rem;
  cursor: pointer;
  border: 1px solid transparent;
}
.btn.ghost {
  background: transparent;
  border-color: var(--border);
  color: var(--text-h);
}
.btn.primary {
  background: var(--accent);
  color: var(--bg);
}
.btn.primary:disabled {
  opacity: 0.6;
  cursor: default;
}
</style>
