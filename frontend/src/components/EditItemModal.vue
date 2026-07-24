<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import type { Item, UpdateItemResp } from "../api/Item";
import { parseCreatedAtInput } from "../utils/date";
import IconCalendar from "~icons/lucide/calendar";

const props = defineProps<{
  item: Item;
  submit: (
    path: string,
    payload: { title: string; created_at: string; author?: string },
  ) => Promise<UpdateItemResp>;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

/** Prefill in the `YYYY-MM-DD HH:mm:ss` format `parseCreatedAtInput` accepts. */
function formatForInput(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;

  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
  );
}

const title = ref(props.item.title ?? props.item.name);
const author = ref(props.item.author ?? "");
const createdAt = ref(formatForInput(props.item.created_at));

const datePicker = ref<HTMLInputElement | null>(null);

/** Extract the `YYYY-MM-DD` portion for the native date picker's value. */
function extractDatePart(input: string): string {
  const match = input.trim().match(/^(\d{4}-\d{2}-\d{2})/);
  return match ? match[1] : "";
}

const openDatePicker = () => {
  datePicker.value?.showPicker();
};

const onDatePicked = (event: Event) => {
  const newDate = (event.target as HTMLInputElement).value;
  if (!newDate) return;

  const timeMatch = createdAt.value.trim().match(/\d{2}:\d{2}:\d{2}$/);
  createdAt.value = timeMatch ? `${newDate} ${timeMatch[0]}` : newDate;
};

const submitting = ref(false);
const error = ref<string | null>(null);

const onKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") emit("close");
};

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});

const submit = async () => {
  if (!title.value.trim()) {
    error.value = "Title is required.";
    return;
  }

  const createdAtInput = createdAt.value.trim();
  const createdAtApiValue = parseCreatedAtInput(createdAtInput);
  if (!createdAtApiValue) {
    error.value = "Date must be YYYY-MM-DD or YYYY-MM-DD HH:mm:ss.";
    return;
  }

  submitting.value = true;
  error.value = null;

  const payload: { title: string; created_at: string; author?: string } = {
    title: title.value.trim(),
    created_at: createdAtApiValue,
  };
  if (author.value.trim()) payload.author = author.value.trim();

  const result = await props.submit(props.item.path, payload);

  submitting.value = false;

  if (result.ok) {
    emit("close");
  } else {
    error.value = result.error;
  }
};
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal" role="dialog" aria-modal="true" aria-label="Edit item">
      <h2 class="modal-title">Edit item</h2>

      <form class="form" @submit.prevent="submit">
        <label>
          <span>Title <em>*</em></span>
          <input v-model="title" type="text" required />
        </label>

        <label>
          <span>Author</span>
          <input v-model="author" type="text" />
        </label>

        <label>
          <span>Date <em>*</em></span>
          <div class="date-field">
            <input v-model="createdAt" type="text" required placeholder="2026-06-17 10:50:00" />
            <button
              type="button"
              class="date-picker-btn"
              aria-label="Pick date"
              @click="openDatePicker"
            >
              <IconCalendar />
            </button>
            <input
              ref="datePicker"
              class="date-picker-hidden"
              type="date"
              tabindex="-1"
              :value="extractDatePart(createdAt)"
              @change="onDatePicked"
            />
          </div>
        </label>

        <p v-if="error" class="form-error">{{ error }}</p>

        <div class="form-actions">
          <button type="button" class="btn ghost" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn primary" :disabled="submitting">
            {{ submitting ? "Saving…" : "Apply" }}
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
.form input {
  font: inherit;
  font-size: 0.9375rem;
  padding: 0.5em 0.65em;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text-h);
}
.form input:focus {
  outline: none;
  border-color: var(--accent);
}
.date-field {
  position: relative;
  display: flex;
}
.date-field input[type='text'] {
  flex: 1;
  padding-right: 2.25rem;
}
.date-picker-btn {
  position: absolute;
  right: 0.25rem;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0.25rem;
  color: var(--text-h);
}
.date-picker-btn svg {
  width: 1.05rem;
  height: 1.05rem;
}
.date-picker-hidden {
  position: absolute;
  inset: 0;
  width: 1px;
  height: 1px;
  opacity: 0;
  pointer-events: none;
  border: none;
  padding: 0;
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
