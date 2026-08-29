<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import type {
  DeleteAttachmentResp,
  DeleteItemResp,
  Item,
  ItemStatus,
  TagFreq,
  UpdateItemResp,
  UpdateStatusResp,
  UpdateTagsResp,
  UploadResp,
} from "../api/Item";
import { type ContentTab, kindHasNote, tabsForKind } from "../api/contentTypes";
import { STATUSES } from "../api/meta";
import { formatDate } from "../utils/format";
import ContentTabs from "./ContentTabs.vue";
import ContentViewer from "./ContentViewer.vue";
import NoteEditor from "./NoteEditor.vue";
import AttachmentsModal from "./AttachmentsModal.vue";
import EditItemModal from "./EditItemModal.vue";
import TagsEditor from "./TagsEditor.vue";
import IconChevronDown from "~icons/lucide/chevron-down";
import IconChevronRight from "~icons/lucide/chevron-right";
import IconEllipsis from "~icons/lucide/ellipsis";
import IconPencil from "~icons/lucide/pencil";
import IconCheck from "~icons/lucide/check";

const props = defineProps<{
  item: Item | null;
  allTags: TagFreq[];
  loadTags: () => Promise<void>;
  updateTags: (path: string, tags: string) => Promise<UpdateTagsResp>;
  updateStatus: (path: string, status: ItemStatus) => Promise<UpdateStatusResp>;
  updateItem: (
    path: string,
    payload: { title: string; created_at: string; author?: string },
  ) => Promise<UpdateItemResp>;
  deleteItem: (path: string) => Promise<DeleteItemResp>;
  deleteAttachment: (path: string, filename: string) => Promise<DeleteAttachmentResp>;
  uploadAttachment: (path: string, file: File) => Promise<UploadResp>;
}>();

const emit = defineEmits<{
  (e: "search-by-tag", value: string): void;
}>();

const statusLabel = (status: ItemStatus | undefined) =>
  STATUSES.find((option) => option.value === status)?.label ?? STATUSES[0].label;

const tabs = computed<ContentTab[]>(() => tabsForKind(props.item?.kind));

const activeTab = ref<ContentTab>(tabs.value[0]);

const select = (tab: ContentTab) => {
  activeTab.value = tab;
  editingContent.value = false;
};

// Toggles the raw editor for the active tab's content (see ContentTabs' pencil button).
const editingContent = ref(false);

const toggleEditContent = () => {
  editingContent.value = !editingContent.value;
};

// Show the title as a link only when there's a real URL (notes/tasks have none).
const hasUrl = computed(() => !!props.item?.url?.trim());

// --- Note sidebar (resizable markdown editor for `note.md`) ---------------
// note/task kinds are their own note and get no separate editor.
// On desktop it sits on the right; on narrow screens it docks to the bottom
// (see the media queries below) and the drag-resizer is hidden.
const showNote = computed(() => !!props.item && kindHasNote(props.item.kind));

const noteCollapsed = ref(false);
const noteWidth = ref(360);
const MIN_NOTE_WIDTH = 220;
const MAX_NOTE_WIDTH = 900;

// Bumped after the note is saved so ContentViewer re-fetches the Note tab.
const noteReloadToken = ref(0);

const onNoteSaved = () => {
  const item = props.item;
  if (item && !item.assets.includes("note.md")) {
    item.assets.push("note.md");
  }
  noteReloadToken.value += 1;
};

let resizeStartX = 0;
let resizeStartWidth = 0;

const onResizeMove = (event: MouseEvent) => {
  // Dragging left (smaller clientX) widens the right-anchored sidebar.
  const delta = resizeStartX - event.clientX;
  const next = Math.min(MAX_NOTE_WIDTH, Math.max(MIN_NOTE_WIDTH, resizeStartWidth + delta));
  noteWidth.value = next;
};

const stopResize = () => {
  window.removeEventListener("mousemove", onResizeMove);
  window.removeEventListener("mouseup", stopResize);
  document.body.style.userSelect = "";
};

const startResize = (event: MouseEvent) => {
  resizeStartX = event.clientX;
  resizeStartWidth = noteWidth.value;
  document.body.style.userSelect = "none";
  window.addEventListener("mousemove", onResizeMove);
  window.addEventListener("mouseup", stopResize);
};

const searchByTag = (tag: string) => {
  const searchTag = /\s/.test(tag) ? `tag:"${tag}"` : `#${tag}`;
  emit("search-by-tag", searchTag);
};

onBeforeUnmount(stopResize);

// Inline status selector.
const editingStatus = ref(false);
const savingStatus = ref(false);
const statusError = ref<string | null>(null);

const toggleStatus = () => {
  if (!props.item) return;
  editingStatus.value = !editingStatus.value;
  statusError.value = null;
};

const chooseStatus = async (status: ItemStatus) => {
  if (!props.item || savingStatus.value) return;
  editingStatus.value = false;

  if (props.item.status === status) return;

  savingStatus.value = true;
  statusError.value = null;

  const response = await props.updateStatus(props.item.path, status);

  savingStatus.value = false;

  if (!response.ok) {
    statusError.value = response.error;
  }
};

const editingTags = ref(false);
const tagsError = ref<string | null>(null);
const savingTags = ref(false);
const suggestions = computed(() => props.allTags.map((t) => t.name));
const tagsFreq = computed(() =>
  Object.fromEntries(props.allTags.map((t) => [t.name, t.count]))
);

const beginEditTags = () => {
  if (!props.item) return;
  tagsError.value = null;
  editingTags.value = true;
  props.loadTags();
};

const doneEditTags = () => {
  editingTags.value = false;
  tagsError.value = null;
};

// Persist the full tag list; the composable maps it back onto the item on success.
const persistTags = async (next: string[]) => {
  if (!props.item || savingTags.value) return;

  savingTags.value = true;
  tagsError.value = null;

  const response = await props.updateTags(props.item.path, next.join(","));

  savingTags.value = false;

  if (!response.ok) {
    tagsError.value = response.error;
  }
};

const addTag = (tag: string) => {
  if (!props.item || props.item.tags.includes(tag)) return;
  persistTags([...props.item.tags, tag]);
};

const removeTag = (tag: string) => {
  if (!props.item) return;
  persistTags(props.item.tags.filter((t) => t !== tag));
};

// Item menu (top-right "..." button): delete for now.
const menuOpen = ref(false);
const deleting = ref(false);
const deleteError = ref<string | null>(null);

const toggleMenu = () => {
  menuOpen.value = !menuOpen.value;
};

const confirmDelete = async () => {
  if (!props.item || deleting.value) return;
  menuOpen.value = false;

  const confirmed = window.confirm(`Delete "${props.item.name}"? This cannot be undone.`);
  if (!confirmed) return;

  deleting.value = true;
  deleteError.value = null;

  const response = await props.deleteItem(props.item.path);

  deleting.value = false;

  if (!response.ok) {
    deleteError.value = response.error;
  }
};

// "Delete attachments" modal: lists real attachments (excludes generated
// content) with per-row delete buttons.
const attachmentsModalOpen = ref(false);

const openAttachmentsModal = () => {
  menuOpen.value = false;
  attachmentsModalOpen.value = true;
};

// "Edit item" modal: title/author/created_at, prefilled from the current item.
const editModalOpen = ref(false);

const openEditModal = () => {
  menuOpen.value = false;
  editModalOpen.value = true;
};

watch(
  () => props.item?.path,
  () => {
    editingTags.value = false;
    tagsError.value = null;
    savingTags.value = false;
    editingStatus.value = false;
    statusError.value = null;
    savingStatus.value = false;
    menuOpen.value = false;
    deleteError.value = null;
    attachmentsModalOpen.value = false;
    editModalOpen.value = false;
    // Reset to the first tab available for the (possibly new) item kind.
    activeTab.value = tabs.value[0];
    editingContent.value = false;
  },
);

// Optimistically mark the asset as present so the tab leaves its "missing"
// state immediately after a successful upload.
const onUploaded = () => {
  const item = props.item;
  if (item && !item.assets.includes(activeTab.value.filename)) {
    item.assets.push(activeTab.value.filename);
  }
};
</script>

<template>
  <section class="item-view">
    <div v-if="!item" class="empty">
      <p>Select an item to read.</p>
    </div>

    <template v-else>
      <div class="item-main">
        <header class="view-header">
          <div class="view-meta-row">
            <span class="view-meta">{{ formatDate(item.created_at) }}</span>

            <div class="status-selector">
              <button
                type="button"
                class="status-badge"
                :class="`status-${item.status ?? 'new'}`"
                :disabled="savingStatus"
                @click="toggleStatus"
              >
              {{ statusLabel(item.status) }}
              <IconChevronDown class="badge-caret" />
            </button>

              <ul v-if="editingStatus" class="status-menu" role="listbox">
                <li v-for="option in STATUSES" :key="option.value">
                  <button
                    type="button"
                    class="status-option"
                    :class="{ active: option.value === (item.status ?? 'new') }"
                    role="option"
                    :aria-selected="option.value === (item.status ?? 'new')"
                    @click="chooseStatus(option.value)"
                  >
                  {{ option.label }}
                </button>
                </li>
              </ul>
            </div>

            <div class="item-menu">
              <button
                type="button"
                class="item-menu-btn"
                aria-label="Item menu"
                title="Item menu"
                :disabled="deleting"
                @click="toggleMenu"
              >
                <IconEllipsis />
              </button>

              <ul v-if="menuOpen" class="item-menu-list" role="menu">
                <li>
                  <button
                    type="button"
                    class="item-menu-option"
                    role="menuitem"
                    @click="openEditModal"
                  >
                  Edit
                </button>
                </li>
                <li>
                  <button
                    type="button"
                    class="item-menu-option"
                    role="menuitem"
                    @click="openAttachmentsModal"
                  >
                  Attachments
                </button>
                </li>
                <li>
                  <button
                    type="button"
                    class="item-menu-option danger"
                    role="menuitem"
                    @click="confirmDelete"
                  >
                  Delete
                </button>
                </li>
              </ul>
            </div>
          </div>

          <p v-if="statusError" class="status-error">{{ statusError }}</p>
          <p v-if="deleteError" class="status-error">{{ deleteError }}</p>

          <h1 class="view-title">
            <a v-if="hasUrl" :href="item.url" target="_blank"
              rel="noreferrer">{{ item.name }}</a>
            <span v-else>{{ item.name }}</span>
          </h1>

          <div class="view-tags-row">
            <template v-if="!editingTags">
              <div v-if="item.tags.length" class="view-tags">
                <span
                  v-for="tag in item.tags"
                  :key="tag"
                  class="tag"
                  title="Add to search"
                  @click="searchByTag(tag)"
                >
                  {{ tag }}
                  <span class="tag-count">{{ tagsFreq[tag] || 1 }}</span>
                </span>
              </div>
              <div v-else class="no-tags">No tags yet.</div>

              <button
                type="button"
                class="edit-tags-btn"
                :disabled="savingTags"
                aria-label="Edit tags"
                title="Edit tags"
                @click="beginEditTags"
              >
                <IconPencil />
              </button>
            </template>

            <button
              v-else
              type="button"
              class="edit-tags-btn done"
              aria-label="Done editing tags"
              title="Done"
              @click="doneEditTags"
            >
              <IconCheck />
            </button>
          </div>

          <div v-if="editingTags" class="tags-editor-wrap">
            <TagsEditor
              :tags="item.tags"
              :suggestions
              :disabled="savingTags"
              @add="addTag"
              @remove="removeTag"
            />
            <p v-if="tagsError" class="tags-error">{{ tagsError }}</p>
          </div>
        </header>

        <ContentTabs
          :tabs="tabs"
          :assets="item.assets"
          :active="activeTab"
          :editing="editingContent"
          @select="select"
          @toggle-edit="toggleEditContent"
        />

        <ContentViewer
          :key="item.path"
          :item="item"
          :tab="activeTab"
          :editing="editingContent"
          :reload-token="noteReloadToken"
          @uploaded="onUploaded"
        />
      </div>

      <!-- Note editor: right sidebar on desktop, bottom panel on mobile. -->
      <template v-if="showNote">
        <div
          v-if="noteCollapsed"
          class="note-strip"
          role="button"
          tabindex="0"
          title="Show note"
          @click="noteCollapsed = false"
          @keydown.enter="noteCollapsed = false"
        >
          <span class="note-strip-label">Note</span>
        </div>

        <template v-else>
          <div
            class="note-resizer"
            title="Drag to resize"
            @mousedown.prevent="startResize"
          />
          <aside class="note-sidebar" :style="{ '--note-width': noteWidth + 'px' }">
            <button
              type="button"
              class="note-collapse"
              title="Hide note"
              @click="noteCollapsed = true"
            >
              <IconChevronRight class="note-collapse-icon" />
            </button>
            <NoteEditor :item="item" @saved="onNoteSaved" />
          </aside>
        </template>
      </template>
    </template>

    <AttachmentsModal
      v-if="attachmentsModalOpen && item"
      :item="item"
      :delete-attachment="deleteAttachment"
      :upload-attachment="uploadAttachment"
      @close="attachmentsModalOpen = false"
    />

    <EditItemModal
      v-if="editModalOpen && item"
      :item="item"
      :submit="updateItem"
      @close="editModalOpen = false"
    />
  </section>
</template>

<style scoped>
.item-view {
  display: flex;
  flex-direction: row;
  flex: 1;
  min-width: 0;
  height: 100%;
  box-sizing: border-box;
  overflow: hidden;
}
.item-main {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
  padding: 1.5rem 1.75rem;
  overflow-y: auto;
}
/* Draggable divider between content and note editor. */
.note-resizer {
  flex: 0 0 0.375rem;
  cursor: col-resize;
  background: var(--border);
  opacity: 0.5;
  transition: opacity 0.2s, background 0.2s;
}
.note-resizer:hover {
  opacity: 1;
  background: var(--accent);
}
.note-sidebar {
  position: relative;
  flex: 0 0 auto;
  width: var(--note-width, 22.5rem);
  min-width: 0;
  height: 100%;
  box-sizing: border-box;
  border-left: 1px solid var(--border);
  background: var(--code-bg);
  overflow: hidden;
}
.note-collapse {
  position: absolute;
  top: 0.875rem;
  right: 0.75rem;
  z-index: 2;
  width: 1.625rem;
  height: 1.625rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text);
  cursor: pointer;
  font-size: 1rem;
}
.note-collapse:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}
/* Collapsed handle: a thin strip to reopen the editor. */
.note-strip {
  flex: 0 0 1.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-left: 1px solid var(--border);
  background: var(--code-bg);
  cursor: pointer;
}
.note-strip:hover {
  background: var(--accent-bg);
  color: var(--accent);
}
.note-strip-label {
  writing-mode: vertical-rl;
  transform: rotate(180deg);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text);
}

/* Narrow screens: stack the note editor under the content instead of beside
   it. The mouse resizer makes no sense there, so it is hidden. */
@media (max-width: 64rem) {
  .item-view {
    flex-direction: column;
  }
  .note-resizer {
    display: none;
  }
  .note-sidebar {
    width: 100%;
    height: 40dvh;
    flex: 0 0 auto;
    border-left: none;
    border-top: 1px solid var(--border);
  }
  .note-collapse-icon {
    transform: rotate(90deg);
  }
  .note-strip {
    flex: 0 0 1.75rem;
    border-left: none;
    border-top: 1px solid var(--border);
  }
  .note-strip-label {
    writing-mode: horizontal-tb;
    transform: none;
  }
}

@media (max-width: 48rem) {
  .item-main {
    /* Clear the floating drawer toggle button. */
    padding: 3.75rem 1rem 1rem;
  }
}

.empty {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  color: var(--text);
}
.view-header {
  margin-bottom: 0.5rem;
}
.view-meta-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-meta {
  font-size: 0.9375rem;
  opacity: 0.8;
}
.status-selector {
  position: relative;
}
.item-menu {
  position: relative;
  margin-left: auto;
}
.item-menu-btn {
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
  font-size: 1rem;
}
.item-menu-btn:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}
.item-menu-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.item-menu-list {
  position: absolute;
  top: calc(100% + 0.25rem);
  right: 0;
  z-index: 20;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  min-width: 8.75rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  box-shadow: var(--shadow);
}
.item-menu-option {
  display: block;
  width: 100%;
  text-align: left;
  font: inherit;
  font-size: 0.875rem;
  padding: 0.4em 0.7em;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-h);
  cursor: pointer;
}
.item-menu-option:hover {
  background: var(--accent-bg);
}
.item-menu-option.danger {
  color: var(--danger);
}
.item-menu-option.danger:hover {
  background: color-mix(in srgb, var(--danger) 12%, transparent);
}
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  font: inherit;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.25em 0.65em;
  border: 1px solid var(--border);
  border-radius: 0.75rem;
  background: var(--accent-bg);
  color: var(--accent);
  cursor: pointer;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}
.badge-caret {
  font-size: 0.75rem;
}
.status-badge:disabled {
  opacity: 0.6;
  cursor: default;
}
.status-badge.status-new {
  color: light-dark(#7c3aed, #a78bfa);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-planned_high {
  color: light-dark(#d66e0d, #f5780b);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-planned {
  color: light-dark(#d78023, #de9922);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-planned_low {
  color: light-dark(#bf9742, #ebca5f);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-in_progress {
  color: light-dark(#2563eb, #60a5fa);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-paused {
  color: light-dark(#6b7280, #9ca3af);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-done {
  color: light-dark(#2e8b57, #4ade80);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-badge.status-rejected {
  color: light-dark(#dc2626, #f87171);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.status-menu {
  position: absolute;
  top: calc(100% + 0.25rem);
  left: 0;
  z-index: 20;
  margin: 0;
  padding: 0.25rem;
  list-style: none;
  min-width: 8.75rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  box-shadow: var(--shadow);
}
.status-option {
  display: block;
  width: 100%;
  text-align: left;
  font: inherit;
  font-size: 0.875rem;
  padding: 0.4em 0.7em;
  border: none;
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-h);
  cursor: pointer;
}
.status-option:hover {
  background: var(--accent-bg);
}
.status-option.active {
  color: var(--accent);
  font-weight: 600;
}
.status-error {
  font-size: 0.8125rem;
  color: var(--danger);
  margin: 0.375rem 0 0;
}
.view-title {
  font-size: 1.6em;
  line-height: 1.2;
  margin: 0.25rem 0 0.5rem;
  letter-spacing: -0.02em;
  overflow-wrap: anywhere;
}
.view-title a {
  color: var(--text-h);
  text-decoration: none;
}
.view-title a:hover {
  text-decoration: underline;
}
.view-tags-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.625rem;
  flex-wrap: wrap;
}
.view-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}
.edit-tags-btn {
  width: 1.875rem;
  height: 1.875rem;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: var(--bg);
  color: var(--text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 0.875rem;
}
.edit-tags-btn:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}
.edit-tags-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.edit-tags-btn.done {
  border-color: var(--accent-border);
  color: var(--accent);
}
.tags-editor-wrap {
  display: grid;
  gap: 0.5rem;
  margin-bottom: 1rem;
}
.tags-error {
  font-size: 0.8125rem;
  color: var(--danger);
}
.no-tags {
  font-size: 0.8125rem;
  color: var(--text);
  opacity: 0.7;
}
.tag {
  font-size: 0.8125rem;
  padding: 0.15em 0.6em;
  border-radius: 0.625rem;
  color: var(--accent);
  background: var(--accent-bg);
  cursor: pointer;
}
.tag-count {
  font-size: 0.75rem;
  margin-left: 0.25em;
  opacity: 0.65;
}
</style>
