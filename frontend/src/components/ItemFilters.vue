<script setup lang="ts">
import { computed, ref } from "vue";
import type { ItemKind, ItemStatus } from "../api/Item";
import { kindMeta, KINDS, STATUSES, statusMeta } from "../api/meta";
import IconCalendar from "~icons/lucide/calendar";

const props = defineProps<{
  kind: ItemKind | null;
  status: ItemStatus | null;
  date: string | null;
}>();

const emit = defineEmits<{
  (e: "kind", kind: ItemKind | null): void;
  (e: "status", status: ItemStatus | null): void;
  (e: "date", date: string | null): void;
  (e: "clear"): void;
}>();

type Panel = "kind" | "status" | "date" | null;
const open = ref<Panel>(null);

const toggle = (panel: Panel) => {
  open.value = open.value === panel ? null : panel;
};
const close = () => {
  open.value = null;
};

const activeKindMeta = computed(() => kindMeta(props.kind));
const activeStatusMeta = computed(() => statusMeta(props.status));

const hasFilters = computed(
  () => props.kind !== null || props.status !== null || props.date !== null,
);

const onKind = (kind: ItemKind | null) => {
  emit("kind", kind);
  close();
};
const onStatus = (status: ItemStatus | null) => {
  emit("status", status);
  close();
};
const onDate = (event: Event) => {
  const value = (event.target as HTMLInputElement).value;
  emit("date", value || null);
};
</script>

<template>
  <div class="filters">
    <div v-if="open" class="filters-backdrop" @click="close"></div>

    <!-- Kind -->
    <div class="filter">
      <button
        type="button"
        class="chip"
        :class="{ active: kind !== null }"
        aria-haspopup="menu"
        :aria-expanded="open === 'kind'"
        @click="toggle('kind')"
      >
        <span
          v-if="activeKindMeta"
          class="swatch"
          :style="{ background: activeKindMeta.color }"
        ></span>
        {{ activeKindMeta?.label ?? "Kind" }}
        <span class="caret">▾</span>
      </button>
      <ul v-if="open === 'kind'" class="menu" role="menu">
        <li>
          <button type="button" role="menuitem" @click="onKind(null)">
            <span class="swatch swatch-any"></span> All
          </button>
        </li>
        <li v-for="option in KINDS" :key="option.value">
          <button type="button" role="menuitem" @click="onKind(option.value)">
            <span class="swatch" :style="{ background: option.color }"></span>
            {{ option.label }}
          </button>
        </li>
      </ul>
    </div>

    <!-- Status -->
    <div class="filter">
      <button
        type="button"
        class="chip"
        :class="{ active: status !== null }"
        aria-haspopup="menu"
        :aria-expanded="open === 'status'"
        @click="toggle('status')"
      >
        <span
          v-if="activeStatusMeta"
          class="swatch"
          :style="{ background: activeStatusMeta.color }"
        ></span>
        {{ activeStatusMeta?.label ?? "Status" }}
        <span class="caret">▾</span>
      </button>
      <ul v-if="open === 'status'" class="menu" role="menu">
        <li>
          <button type="button" role="menuitem" @click="onStatus(null)">
            <span class="swatch swatch-any"></span> All
          </button>
        </li>
        <li v-for="option in STATUSES" :key="option.value">
          <button type="button" role="menuitem" @click="onStatus(option.value)">
            <span class="swatch" :style="{ background: option.color }"></span>
            {{ option.label }}
          </button>
        </li>
      </ul>
    </div>

    <!-- Date -->
    <div class="filter">
      <button
        type="button"
        class="chip icon-chip"
        :class="{ active: date !== null }"
        aria-haspopup="dialog"
        :aria-expanded="open === 'date'"
        title="Filter by date"
        @click="toggle('date')"
      >
        <IconCalendar />
      </button>
      <div v-if="open === 'date'" class="menu date-menu" role="dialog">
        <input
          class="date-input"
          type="date"
          :value="date ?? ''"
          @input="onDate"
        />
        <a
          v-if="date"
          class="clear-date"
          href="#"
          @click.prevent="emit('date', null)"
        >Clear date</a>
      </div>
    </div>

    <a
      v-if="hasFilters"
      class="clear-all"
      href="#"
      @click.prevent="emit('clear')"
    >Clear all</a>
  </div>
</template>

<style scoped>
.filters {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}
.filters-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10;
}
.filter {
  position: relative;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  font: inherit;
  font-size: 0.8125rem;
  padding: 0.4em 0.75em;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  color: var(--text-h);
  cursor: pointer;
  transition: border-color 0.2s;
}
.chip:hover {
  border-color: var(--accent-border);
}
.chip.active {
  border-color: var(--accent);
}
.icon-chip {
  padding: 0.3em 0.55em;
  font-size: 0.9375rem;
}
.caret {
  font-size: 0.625rem;
  opacity: 0.6;
}
.swatch {
  width: 0.5625rem;
  height: 0.5625rem;
  border-radius: 50%;
  flex-shrink: 0;
}
.swatch-any {
  background: var(--text);
  opacity: 0.4;
}
.menu {
  position: absolute;
  top: calc(100% + 0.375rem);
  left: 0;
  z-index: 11;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  min-width: 9.375rem;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  box-shadow: var(--shadow);
}
.menu button {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  width: 100%;
  text-align: left;
  font: inherit;
  font-size: 0.875rem;
  padding: 0.5em 0.85em;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-h);
  cursor: pointer;
}
.menu button:hover {
  background: var(--accent-bg);
  color: var(--accent);
}
.date-menu {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
}
.date-input {
  font: inherit;
  font-size: 0.875rem;
  padding: 0.4em 0.55em;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text-h);
}
.clear-date {
  font-size: 0.8125rem;
  color: var(--accent);
  text-decoration: underline;
}
.clear-all {
  margin-left: auto;
  font-size: 0.8125rem;
  color: var(--accent);
  text-decoration: none;
}
.clear-all:hover {
  text-decoration: underline;
}
</style>
