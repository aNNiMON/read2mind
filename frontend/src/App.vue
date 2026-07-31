<script setup lang="ts">
import { ref } from "vue";
import type { Item, ItemKind } from "./api/Item";
import useItems from "./composables/useItems";
import "./composables/useSettings";
import ItemList from "./components/ItemList.vue";
import ItemView from "./components/ItemView.vue";
import AddItemModal from "./components/AddItemModal.vue";
import SettingsModal from "./components/SettingsModal.vue";
import IconMenu from "~icons/lucide/menu";

const {
  items,
  loading,
  loadingMore,
  hasMore,
  loadMore,
  error,
  allTags,
  loadTags,
  activeKind,
  activeStatus,
  activeDate,
  selected,
  select,
  search,
  setKind,
  setStatus,
  setDate,
  clearFilters,
  addItem,
  updateItem,
  updateItemTags,
  updateItemStatus,
  deleteItem,
  deleteAttachment,
  uploadAttachment,
} = useItems();

const addKind = ref<ItemKind | null>(null);
const showSettings = ref(false);

// On mobile the item list is an off-canvas drawer.
const drawerOpen = ref(false);

const onSelect = (item: Item) => {
  select(item);
  drawerOpen.value = false;
};
</script>

<template>
  <button
    class="drawer-toggle"
    type="button"
    title="Show items"
    aria-label="Show item list"
    @click="drawerOpen = true"
  >
    <IconMenu />
  </button>
  <div v-if="drawerOpen" class="drawer-backdrop" @click="drawerOpen = false"></div>

  <ItemList
    :class="{ 'drawer-open': drawerOpen }"
    :items="items"
    :loading="loading"
    :loading-more="loadingMore"
    :has-more="hasMore"
    :error="error"
    :selected="selected"
    :kind="activeKind"
    :status="activeStatus"
    :date="activeDate"
    :all-tags="allTags"
    @select="onSelect"
    @settings="showSettings = true"
    @add="addKind = $event"
    @search="search"
    @kind="setKind"
    @status="setStatus"
    @date="setDate"
    @clear="clearFilters"
    @load-more="loadMore"
  />

  <ItemView
    :item="selected"
    :all-tags="allTags"
    :load-tags="loadTags"
    :update-tags="updateItemTags"
    :update-status="updateItemStatus"
    :update-item="updateItem"
    :delete-item="deleteItem"
    :delete-attachment="deleteAttachment"
    :upload-attachment="uploadAttachment"
  />

  <AddItemModal
    v-if="addKind"
    :kind="addKind"
    :submit="addItem"
    @close="addKind = null"
  />

  <SettingsModal v-if="showSettings" @close="showSettings = false" />
</template>

<style scoped>
/* Floating hamburger, only shown on mobile where the list is a drawer. */
.drawer-toggle {
  display: none;
  position: fixed;
  top: 0.75rem;
  left: 0.75rem;
  z-index: 40;
  width: 2.5rem;
  height: 2.5rem;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  color: var(--text-h);
  box-shadow: var(--shadow);
  cursor: pointer;
}
.drawer-backdrop {
  display: none;
  position: fixed;
  inset: 0;
  z-index: 49;
  background: rgba(0, 0, 0, 0.4);
}

@media (max-width: 48rem) {
  .drawer-toggle {
    display: inline-flex;
  }
  .drawer-backdrop {
    display: block;
  }
}
</style>
