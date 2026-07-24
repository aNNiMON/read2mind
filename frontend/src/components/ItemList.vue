<script setup lang="ts">
import { ref } from "vue";
import type { Item, ItemKind, ItemStatus } from "../api/Item";
import { KINDS } from "../api/meta";
import ItemRow from "./ItemRow.vue";
import ItemFilters from "./ItemFilters.vue";
import IconSearch from "~icons/lucide/search";
import IconSettings from "~icons/lucide/settings";
import IconPlus from "~icons/lucide/plus";

defineProps<{
  items: Item[];
  loading: boolean;
  loadingMore?: boolean;
  hasMore?: boolean;
  error: string | null;
  selected: Item | null;
  kind: ItemKind | null;
  status: ItemStatus | null;
  date: string | null;
}>();

const emit = defineEmits<{
  (e: "select", item: Item): void;
  (e: "search", query: string): void;
  (e: "settings"): void;
  (e: "add", kind: ItemKind): void;
  (e: "kind", kind: ItemKind | null): void;
  (e: "status", status: ItemStatus | null): void;
  (e: "date", date: string | null): void;
  (e: "clear"): void;
  (e: "load-more"): void;
}>();

// Fire 'load-more' once the list is scrolled near its bottom.
const onScroll = (event: Event) => {
  const el = event.target as HTMLElement;
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {
    emit("load-more");
  }
};

const query = ref("");
const showAddMenu = ref(false);

const onSearch = () => emit("search", query.value);

const onAdd = (kind: ItemKind) => {
  showAddMenu.value = false;
  emit("add", kind);
};
</script>

<template>
  <aside class="item-list">
    <header class="list-header">
      <div class="header-row">
        <span class="logo" aria-hidden="true">R₂M</span>
        <form class="search" @submit.prevent="onSearch">
          <input
            v-model="query"
            class="search-input"
            type="search"
            placeholder="search"
            aria-label="Search items by tag"
            @search="onSearch"
          />
        </form>
        <button class="icon-btn" type="button" title="Search" @click="onSearch">
          <IconSearch />
        </button>
        <button class="icon-btn" type="button" title="Settings" @click="emit('settings')">
          <IconSettings />
        </button>
        <div class="add-wrap">
          <button
            class="icon-btn"
            type="button"
            title="Add item"
            aria-haspopup="menu"
            :aria-expanded="showAddMenu"
            @click="showAddMenu = !showAddMenu"
          >
            <IconPlus />
          </button>
          <template v-if="showAddMenu">
            <div class="add-menu-backdrop" @click="showAddMenu = false"></div>
            <ul class="add-menu" role="menu">
              <li v-for="option in KINDS" :key="option.value">
                <button type="button" role="menuitem" @click="onAdd(option.value)">
                <span class="swatch" :style="{ background: option.color }"></span>
                {{ option.label }}
              </button>
              </li>
            </ul>
          </template>
        </div>
      </div>

      <ItemFilters
        :kind="kind"
        :status="status"
        :date="date"
        @kind="emit('kind', $event)"
        @status="emit('status', $event)"
        @date="emit('date', $event)"
        @clear="emit('clear')"
      />
    </header>

    <div class="list-body" @scroll="onScroll">
      <p v-if="error" class="list-note error">{{ error }}</p>
      <p v-if="loading" class="list-note">Loading…</p>
      <p v-else-if="!items.length" class="list-note">No items found.</p>

      <ItemRow
        v-for="item in items"
        :key="item.path"
        :item="item"
        :active="selected?.path === item.path"
        @click="emit('select', item)"
      />

      <p v-if="loadingMore" class="list-note">Loading more…</p>
    </div>
  </aside>
</template>

<style scoped>
.item-list {
  display: flex;
  flex-direction: column;
  width: 22.5rem;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
  height: 100%;
  box-sizing: border-box;
  background: var(--bg);
}

/* Mobile: off-canvas drawer, slid in via .drawer-open (set by App.vue). */
@media (max-width: 48rem) {
  .item-list {
    position: fixed;
    top: 0;
    bottom: 0;
    left: 0;
    z-index: 50;
    width: min(22.5rem, 85vw);
    height: 100dvh;
    transform: translateX(-100%);
    transition: transform 0.25s ease;
    box-shadow: none;
  }
  .item-list.drawer-open {
    transform: translateX(0);
    box-shadow: var(--shadow);
  }
}

.list-header {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
}
.header-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.logo {
  flex-shrink: 0;
}
.search {
  flex: 1;
  min-width: 0;
}
.search-input {
  width: 100%;
  box-sizing: border-box;
  font: inherit;
  font-size: 0.9375rem;
  padding: 0.375em 0.625em;
  border: none;
  border-bottom: 1px solid var(--border);
  background: transparent;
  color: var(--text-h);
}
.search-input:focus {
  outline: none;
  border-bottom-color: var(--accent);
}
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  font-size: 1.125rem;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text);
  cursor: pointer;
  flex-shrink: 0;
  transition: background 0.2s, color 0.2s;
}
.icon-btn:hover {
  background: var(--accent-bg);
  color: var(--accent);
}
.add-wrap {
  position: relative;
  flex-shrink: 0;
}
.add-menu-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10;
}
.add-menu {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: 11;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  min-width: 8.75rem;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  box-shadow: var(--shadow);
}
.add-menu li {
  margin: 0;
}
.add-menu button {
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
.add-menu button:hover {
  background: var(--accent-bg);
  color: var(--accent);
}
.swatch {
  width: 0.5625rem;
  height: 0.5625rem;
  border-radius: 50%;
  flex-shrink: 0;
}
.list-body {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  overflow-y: auto;
}
.list-note {
  font-size: 0.875rem;
  padding: 0.25rem 0.125rem;
}
.list-note.error {
  color: var(--danger);
}
</style>
