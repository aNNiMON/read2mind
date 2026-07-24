<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import IconX from "~icons/lucide/x";

const props = defineProps<{
  tags: string[];
  suggestions: string[];
  disabled?: boolean;
}>();

const emit = defineEmits<{
  add: [tag: string];
  remove: [tag: string];
}>();

const query = ref("");
const open = ref(false);
const activeIndex = ref(0);
const inputEl = ref<HTMLInputElement | null>(null);

// Suggestions not yet applied to this item, filtered by the current query.
const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  return props.suggestions
    .filter((tag) => !props.tags.includes(tag))
    .filter((tag) => !q || tag.toLowerCase().includes(q));
});

const focusInput = () => {
  if (props.disabled) return;
  open.value = true;
  inputEl.value?.focus();
};

// Set while an add is in flight so the suggestions reopen once it settles;
// left false for removes so Backspace doesn't pop the dropdown back up.
const pendingReopen = ref(false);
// Set right before we programmatically refocus after a remove, so the
// resulting focus event doesn't reopen the dropdown on its own.
const suppressAutoOpen = ref(false);

const addTag = (raw: string) => {
  const tag = raw.trim();
  if (!tag || props.tags.includes(tag)) return;
  pendingReopen.value = true;
  emit("add", tag);
  query.value = "";
  activeIndex.value = 0;
  nextTick(() => inputEl.value?.focus());
};

const removeTag = (tag: string) => {
  if (props.disabled) return;
  emit("remove", tag);
};

// The input gets disabled (and loses focus) while a save is in flight;
// bring focus back once it's re-enabled so add/remove don't kick the user out.
watch(
  () => props.disabled,
  (disabled, wasDisabled) => {
    if (!disabled && wasDisabled) {
      if (!pendingReopen.value) suppressAutoOpen.value = true;
      pendingReopen.value = false;
      nextTick(() => inputEl.value?.focus());
    }
  },
);

const onFocus = () => {
  if (suppressAutoOpen.value) {
    suppressAutoOpen.value = false;
    return;
  }
  open.value = true;
};

const onInput = () => {
  open.value = true;
  activeIndex.value = 0;
};

const onEnter = () => {
  const match = filtered.value[activeIndex.value];
  addTag(match ?? query.value);
};

const onBackspace = () => {
  if (query.value === "" && props.tags.length) {
    removeTag(props.tags[props.tags.length - 1]);
  }
};

const moveActive = (delta: number) => {
  open.value = true;
  const count = filtered.value.length;
  if (!count) return;
  activeIndex.value = (activeIndex.value + delta + count) % count;
};

// Delay so a suggestion click registers before the dropdown closes. Also
// guards against the forced blur that disabling the input triggers while
// saving: if focus has already been restored by then, leave it open.
const onBlur = () => {
  setTimeout(() => {
    if (document.activeElement !== inputEl.value) {
      open.value = false;
    }
  }, 120);
};
</script>

<template>
  <div class="tags-editor" :class="{ disabled }">
    <div class="tags-box" @mousedown.self.prevent="focusInput">
      <span v-for="tag in tags" :key="tag" class="tag chip">
        {{ tag }}
        <button
          type="button"
          class="chip-remove"
          :disabled="disabled"
          aria-label="Remove tag"
          @click="removeTag(tag)"
        >
          <IconX />
        </button>
      </span>
      <input
        ref="inputEl"
        v-model="query"
        class="tag-input"
        type="text"
        :disabled="disabled"
        :placeholder="tags.length ? '' : 'Add tags…'"
        @focus="onFocus"
        @input="onInput"
        @blur="onBlur"
        @keydown.enter.prevent="onEnter"
        @keydown.delete="onBackspace"
        @keydown.down.prevent="moveActive(1)"
        @keydown.up.prevent="moveActive(-1)"
        @keydown="(e) => (e.key === ',' && (e.preventDefault(), addTag(query)))"
      />
    </div>

    <ul v-if="open && filtered.length" class="suggestions">
      <li
        v-for="(tag, index) in filtered"
        :key="tag"
        class="suggestion"
        :class="{ active: index === activeIndex }"
        @mousedown.prevent="addTag(tag)"
        @mouseenter="activeIndex = index"
      >
        {{ tag }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.tags-editor {
  position: relative;
}
.tags-box {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.375rem;
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  cursor: text;
}
.tags-box:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-bg);
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}
.chip-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
  padding: 0;
  opacity: 0.7;
  font-size: 0.75rem;
}
.chip-remove:hover {
  opacity: 1;
}
.chip-remove:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}
.tag-input {
  flex: 1;
  min-width: 6rem;
  border: none;
  outline: none;
  background: transparent;
  color: var(--text-h);
  font: inherit;
  font-size: 0.875rem;
  padding: 0.15rem 0.1rem;
}
.suggestions {
  position: absolute;
  z-index: 20;
  top: calc(100% + 0.25rem);
  left: 0;
  right: 0;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  max-height: 14rem;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
}
.suggestion {
  padding: 0.45rem 0.6rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  color: var(--text-h);
  cursor: pointer;
}
.suggestion.active {
  background: var(--accent-bg);
  color: var(--accent);
}
.tags-editor.disabled {
  opacity: 0.6;
}
</style>
