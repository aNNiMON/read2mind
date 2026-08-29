<script setup lang="ts">
import { computed, ref } from "vue";
import type { Item, ItemKind, ItemStatus, TagFreq } from "../api/Item";
import { KINDS } from "../api/meta";
import ItemRow from "./ItemRow.vue";
import ItemFilters from "./ItemFilters.vue";
import IconSearch from "~icons/lucide/search";
import IconSettings from "~icons/lucide/settings";
import IconPlus from "~icons/lucide/plus";

const props = defineProps<{
  items: Item[];
  loading: boolean;
  loadingMore?: boolean;
  hasMore?: boolean;
  error: string | null;
  selected: Item | null;
  kind: ItemKind | null;
  status: ItemStatus | null;
  date: string | null;
  searchText: string;
  allTags?: TagFreq[];
}>();

const emit = defineEmits<{
  (e: "select", item: Item): void;
  (e: "search", query: string): void;
  (e: "update:searchText", value: string): void;
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

const query = computed({
  get: () => props.searchText,
  set: (value: string) => emit("update:searchText", value),
});
const showAddMenu = ref(false);
const searchInputEl = ref<HTMLInputElement | null>(null);
const suggestOpen = ref(false);
const activeIndex = ref(0);

// Tag suggestion activated by # or `tag:`.
const activeTag = computed(() => query.value.match(/(^|\s)(-?)(tag:|#)(?:"[^"]*|[^\s]*)$/i));

// Tags already present before the active tag expression are excluded from suggestions.
const usedTags = computed(() => {
  const beforeActive = activeTag.value
    ? query.value.slice(0, -activeTag.value[0].length)
    : query.value;
  const tagPattern = /(?:^|\s)-?(?:#|tag:)(?:"([^"]*)"|([^\s]+))/gi;
  const tags: string[] = [];
  let match: RegExpExecArray | null;

  while ((match = tagPattern.exec(beforeActive)) !== null) {
    const tag = (match[1] ?? match[2] ?? "").trim();
    if (tag) tags.push(tag);
  }

  return tags;
});

const suggestions = computed(() => {
  const match = activeTag.value;
  if (!match) return [];

  const q = match[0]
    .replace(/^\s*-?(tag:|#)/i, "")
    .replace(/^"/, "")
    .toLowerCase();
  return (props.allTags ?? [])
    .map((t) => t.name)
    .filter((tag) => !usedTags.value.includes(tag))
    .filter((tag) => !q || tag.toLowerCase().includes(q));
});

const applySuggestion = (tag: string) => {
  const match = activeTag.value;
  if (!match) return;

  const prefix = match[1];
  const negate = match[2];
  let tagType = match[3];
  const hasSpace = /\s/.test(tag);
  let formattedTag = tag;
  if (hasSpace) {
    // Disallow #"tag with space", switch to tag:"tag with space" instead.
    tagType = "tag:";
    formattedTag = `"${tag}"`;
  }
  query.value = query.value.slice(0, -match[0].length) +
    `${prefix}${negate}${tagType}${formattedTag} `;
  activeIndex.value = 0;
  searchInputEl.value?.focus();
};

const onSearchInput = () => {
  suggestOpen.value = Boolean(activeTag.value);
  activeIndex.value = 0;
};

const moveActive = (delta: number) => {
  suggestOpen.value = true;
  const count = suggestions.value.length;
  if (!count) return;
  activeIndex.value = (activeIndex.value + delta + count) % count;
};

const onSearchEnter = () => {
  if (suggestOpen.value && suggestions.value.length) {
    applySuggestion(suggestions.value[activeIndex.value]);
    return;
  }
  onSearch();
};

const onSearchBlur = () => {
  setTimeout(() => {
    if (document.activeElement !== searchInputEl.value) {
      suggestOpen.value = false;
    }
  }, 120);
};

const onSearch = () => {
  suggestOpen.value = false;
  emit("search", query.value);
};

const onAdd = (kind: ItemKind) => {
  showAddMenu.value = false;
  emit("add", kind);
};
</script>

<template>
  <aside class="item-list">
    <header class="list-header">
      <div class="header-row">
        <svg class="logo" aria-hidden="true" viewBox="0 0 48 48"
          xmlns="http://www.w3.org/2000/svg">
          <path fill="#bdbdbd" stroke="#5e5e5e" stroke-linecap="round" stroke-linejoin="round"
            stroke-width="2.18"
            d="M3.968 28.627c0 5.042 3.535 8.068 8.248 8.068 1.179 3.026 4.714 5.043 8.249 5.043s6.48-1.513 8.248-4.034c1.768.504 4.125.504 5.892-.505s2.357-3.025 1.768-4.538c4.124-.504 7.66-3.53 7.66-7.06H8.68c-2.356 0-4.713 1.21-4.713 3.026M3.968 19.373c0-5.042 3.535-8.068 8.248-8.068 1.179-3.026 4.714-5.043 8.249-5.043s6.48 1.513 8.248 4.034c1.768-.504 4.125-.504 5.892.505s2.357 3.025 1.768 4.538c4.124.504 7.66 3.53 7.66 7.06H8.68c-2.356 0-4.713-1.21-4.713-3.026" />
          <path class="logo-page" stroke-linejoin="round" stroke-width="1.594"
            d="m22.363 14.168 11.309 3.296v15.383l-11.31-3.297Z" />
          <path class="logo-page" stroke-linejoin="round" stroke-width="1.594"
            d="m22.383 14.168-11.309 3.296v15.383l11.31-3.297z" />
          <path fill="#5f5245"
            d="m20.478 17.762-7.517 2.217-.012 1.412 7.516-2.258zM20.478 21.985l-7.517 2.218-.012 1.411 7.516-2.257zM20.478 25.981l-7.517 2.217-.012 1.412 7.516-2.258z"
            overflow="visible" />
          <path fill="#5f5245"
            d="m24.17 17.774 7.346 2.218.012 1.411-7.345-2.257zM24.17 21.998l7.346 2.217.012 1.412-7.345-2.258zM24.17 25.993l7.346 2.218.012 1.412-7.345-2.258z"
            overflow="visible" />
        </svg>
        <form class="search" @submit.prevent="onSearchEnter">
          <input
            ref="searchInputEl"
            v-model="query"
            class="search-input"
            type="search"
            placeholder="search"
            aria-label="Search items by keyword or tag"
            autocomplete="off"
            @input="onSearchInput"
            @focus="suggestOpen = Boolean(activeTag)"
            @blur="onSearchBlur"
            @search="onSearch"
            @keydown.down.prevent="moveActive(1)"
            @keydown.up.prevent="moveActive(-1)"
            @keydown.esc="suggestOpen = false"
          />
          <ul v-if="suggestOpen && suggestions.length" class="search-suggestions">
            <li
              v-for="(tag, index) in suggestions"
              :key="tag"
              class="suggestion"
              :class="{ active: index === activeIndex }"
              @mousedown.prevent="applySuggestion(tag)"
              @mouseenter="activeIndex = index"
            >
              {{ tag }}
            </li>
          </ul>
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
        :has-search="Boolean(query.trim())"
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
  width: 2.125rem;
  height: 2.125rem;
  display: block;
}
.logo-page {
  fill: var(--accent);
  stroke: #403734;
}
.search {
  position: relative;
  flex: 1;
  min-width: 0;
}
.search-suggestions {
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
.search-suggestions .suggestion {
  padding: 0.45rem 0.6rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
  color: var(--text-h);
  cursor: pointer;
}
.search-suggestions .suggestion.active {
  background: var(--accent-bg);
  color: var(--accent);
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
