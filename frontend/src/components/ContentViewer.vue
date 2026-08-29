<script setup lang="ts">
import { computed, defineAsyncComponent, ref, toRef, watch } from "vue";
import { marked } from "marked";
import DOMPurify from "dompurify";
import markedCallouts from "../utils/markdown/callouts";
import ItemsApi from "../api/ItemsApi";
import type { Item } from "../api/Item";
import type { ContentTab } from "../api/contentTypes";
import useItemContent from "../composables/useItemContent";
import UploadArea from "./UploadArea.vue";

// markmap-lib/markmap-view pull in d3 and are only needed for the mindmap tab,
// so keep them out of the main bundle.
const MindmapViewer = defineAsyncComponent(() => import("./MindmapViewer.vue"));
import IconSparkles from "~icons/lucide/sparkles";
import IconRefreshCw from "~icons/lucide/refresh-cw";
import IconMinimize2 from "~icons/lucide/minimize-2";
import IconMaximize2 from "~icons/lucide/maximize-2";
import IconScan from "~icons/lucide/scan";

const props = defineProps<{
  item: Item;
  tab: ContentTab;
  // Bumped by the parent to force a re-fetch (e.g. after the note is saved).
  reloadToken?: number;
  // Toggled by the pencil button in ContentTabs for editable markdown tabs.
  editing?: boolean;
}>();

const emit = defineEmits<{
  (e: "uploaded"): void;
}>();

const itemsApi = new ItemsApi();

const { body, loading, missing, error, assetUrl, reload } = useItemContent(
  toRef(props, "item"),
  toRef(props, "tab"),
);

// A URL is "relative to the item" only when it has no scheme, isn't
// protocol/root-relative, and isn't an anchor — so bare `image.png` resolves
// against the item's asset directory while `/data/...` and `http://…` stay put.
const isItemRelative = (href: string) => !/^([a-z][a-z0-9+.-]*:|\/\/|\/|#)/i.test(href);

// Resolve relative image sources against the current item's asset directory so
// notes can reference attachments as `![](image.png)` regardless of the SPA URL
// (and survive the item being renamed).
const buildRenderer = (base: string) => {
  const renderer = new marked.Renderer();
  const renderImage = renderer.image.bind(renderer);
  renderer.image = (token) =>
    renderImage({
      ...token,
      href: isItemRelative(token.href)
        ? base + token.href.split("/").map(encodeURIComponent).join("/")
        : token.href,
    });
  return renderer;
};

const renderedMarkdown = computed(() => {
  if (props.tab.kind !== "markdown") return "";
  const base = props.item.assetPath ? itemsApi.assetBaseUrl(props.item.assetPath) : "";
  const html = marked
    .use(markedCallouts())
    .parse(body.value, {
      renderer: base ? buildRenderer(base) : undefined,
      gfm: true,
      breaks: true,
    }) as string;
  return DOMPurify.sanitize(html);
});

const uploadPath = computed(() => props.item.assetPath ?? props.item.path);

// --- Raw editor for editable markdown tabs (toggled by the tab's pencil icon) ---
const isEditingContent = computed(() => !!props.tab.editable && !!props.editing);

const editContent = ref("");
const savingContent = ref(false);
const contentSaveError = ref<string | null>(null);

// Keep the editor's text in sync with freshly loaded content whenever it changes
// and the editor isn't mid-edit (avoids clobbering unsaved keystrokes on reload).
watch(
  () => [body.value, isEditingContent.value],
  () => {
    if (isEditingContent.value) editContent.value = body.value;
  },
  { immediate: true },
);

const saveContent = async () => {
  if (savingContent.value || editContent.value === body.value) return;

  savingContent.value = true;
  contentSaveError.value = null;

  const file = new File([editContent.value], props.tab.filename, { type: "text/markdown" });
  const response = await itemsApi.uploadAttachment(uploadPath.value, file);

  savingContent.value = false;

  if (response.ok) {
    reload();
    return;
  }

  contentSaveError.value = response.error;
};

// Summary/MindMap are AI-generated rather than uploaded.
const isAiTab = computed(() => !!props.tab.aiFeature);

const mindmapRef = ref<InstanceType<typeof MindmapViewer>>();

const generating = ref(false);
const generateError = ref<string | null>(null);

const generate = async () => {
  const feature = props.tab.aiFeature;
  if (!feature || generating.value) return;

  generating.value = true;
  generateError.value = null;

  const response = await itemsApi.generateAi(uploadPath.value, feature);

  generating.value = false;

  if (response.ok) {
    emit("uploaded");
    reload();
  } else {
    generateError.value = response.error;
  }
};

const onUploaded = () => {
  emit("uploaded");
  reload();
};

// Re-fetch when the parent signals external changes to the current asset.
// Only relevant when the Note tab is the one being viewed — a note save
// shouldn't disturb whatever other tab (e.g. Article) is currently open.
watch(
  () => props.reloadToken,
  () => {
    if (props.tab.filename === "note.md") reload();
  },
);

// Clear any stale generation error when switching items/tabs.
watch(
  () => [props.item.path, props.tab.key],
  () => {
    generateError.value = null;
  },
);
</script>

<template>
  <div class="content-viewer">
    <p v-if="loading" class="state">Loading…</p>
    <p v-else-if="error" class="state error">{{ error }}</p>

    <!-- AI-generated tabs (Summary / MindMap) offer generation, not upload. -->
    <div v-else-if="missing && isAiTab" class="ai-generate">
      <h2 class="no-content">No {{ tab.label }}</h2>
      <p class="hint">Generate the {{ tab.label.toLowerCase() }} with AI.</p>

      <button class="ai-btn" type="button" :disabled="generating" @click="generate">
        <IconSparkles />
        {{ generating ? "Generating…" : `Generate ${tab.label}` }}
      </button>

      <p v-if="generateError" class="ai-error">{{ generateError }}</p>
    </div>

    <UploadArea
      v-else-if="missing"
      :path="uploadPath"
      :tab="tab"
      @uploaded="onUploaded"
    />

    <iframe
      v-else-if="tab.kind === 'html'"
      class="html-frame"
      :srcdoc="body"
      sandbox=""
      title="Item content"
    />

    <!-- Markdown is produced by `marked` from the user's own saved content. -->
    <div v-else-if="tab.kind === 'markdown'" class="markdown-pane">
      <!-- AI tabs get a toolbar to regenerate the existing content. -->
      <div v-if="isAiTab" class="ai-toolbar">
        <button class="regen-btn" type="button" :disabled="generating" @click="generate">
          <IconRefreshCw />
          {{ generating ? "Regenerating…" : "Regenerate" }}
        </button>
        <span v-if="generateError" class="ai-error inline">{{ generateError }}</span>
      </div>

      <textarea
        v-if="isEditingContent"
        v-model="editContent"
        class="markdown-input"
        spellcheck="false"
        @blur="saveContent"
      />
      <div v-else class="markdown" v-html="renderedMarkdown" />

      <p v-if="contentSaveError" class="ai-error">{{ contentSaveError }}</p>
    </div>

    <div v-else-if="tab.kind === 'mindmap'" class="mindmap-pane">
      <div v-if="isAiTab" class="ai-toolbar">
        <button class="regen-btn" type="button" :disabled="generating" @click="generate">
          <IconRefreshCw />
          {{ generating ? "Regenerating…" : "Regenerate" }}
        </button>

        <button class="tool-btn" type="button" title="Collapse all"
          @click="mindmapRef?.collapseAll()">
          <IconMinimize2 />
          Collapse all
        </button>

        <button class="tool-btn" type="button" title="Expand all"
          @click="mindmapRef?.expandAll()">
          <IconMaximize2 />
          Expand all
        </button>

        <button class="tool-btn" type="button" title="Fit to viewport"
          @click="mindmapRef?.fit()">
          <IconScan />
          Fit
        </button>

        <span v-if="generateError" class="ai-error inline">{{ generateError }}</span>
      </div>

      <MindmapViewer ref="mindmapRef" :markdown="body" />
    </div>

    <div v-else-if="tab.kind === 'audio'" class="audio">
      <audio :src="assetUrl()" controls />
    </div>
  </div>
</template>

<style scoped>
.content-viewer {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
.state {
  font-size: 0.9375rem;
  padding: 0.5rem 0;
}
.state.error {
  color: var(--danger);
}
.html-frame {
  flex: 1;
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: #fff;
}
.markdown-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
.mindmap-pane {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
.markdown {
  line-height: 155%;
  color: var(--text-h);
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
.markdown-input {
  flex: 1;
  min-height: 0;
  resize: none;
  font-family: var(--mono);
  font-size: 0.875rem;
  line-height: 150%;
  color: var(--text-h);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  padding: 0.75rem;
  box-sizing: border-box;
}
.markdown-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-bg);
}
/* Toolbar above AI-generated content (Summary / MindMap). */
.ai-toolbar {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  flex-wrap: wrap;
  padding: 0 0 0.625rem;
  margin-bottom: 0.625rem;
  border-bottom: 1px solid var(--border);
}
.regen-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  font: inherit;
  font-size: 0.8125rem;
  padding: 0.35em 0.7em;
  border: 1px solid var(--accent-border);
  border-radius: 0.375rem;
  background: var(--accent-bg);
  color: var(--accent);
  cursor: pointer;
}
.regen-btn:disabled {
  opacity: 0.6;
  cursor: default;
}
.tool-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  font: inherit;
  font-size: 0.8125rem;
  padding: 0.35em 0.7em;
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  background: transparent;
  color: var(--text-h);
  cursor: pointer;
}
.tool-btn:hover {
  border-color: var(--accent-border);
  color: var(--accent);
}
/* AI generation placeholder shown when the asset is missing. */
.ai-generate {
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
}
.ai-generate .no-content {
  font-size: 2rem;
  margin: 0;
}
.ai-generate .hint {
  font-size: 0.875rem;
  color: var(--text);
}
.ai-btn {
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
.ai-btn:disabled {
  opacity: 0.6;
  cursor: default;
}
.ai-error {
  font-size: 0.875rem;
  color: var(--danger);
}
.ai-error.inline {
  font-size: 0.8125rem;
}
.markdown :deep(pre) {
  background: var(--code-bg);
  padding: 0.75rem;
  border-radius: 0.5rem;
  overflow-x: auto;
}
.markdown :deep(code) {
  font-family: var(--mono);
}
.markdown :deep(img) {
  max-width: 100%;
}
.markdown :deep(a) {
  color: var(--accent);
}
.markdown :deep(p) {
  margin-bottom: 1.4rem;
}
.markdown :deep(.markdown-callout) {
  --callout-color: var(--accent);
  margin: 1rem 0;
  padding: 0.75rem 1rem;
  border-left: 0.25rem solid var(--callout-color);
  border-radius: 0 0.5rem 0.5rem 0;
  background: color-mix(in srgb, var(--callout-color) 10%, transparent);
}
.markdown :deep(.markdown-callout-title) {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
  color: var(--callout-color);
  font-weight: 700;
}
.markdown :deep(.markdown-callout-title svg) {
  width: 1.1em;
  height: 1.1em;
  flex: none;
}
.markdown :deep(.markdown-callout > :last-child) {
  margin-bottom: 0;
}
.markdown :deep(.markdown-callout-warning) {
  --callout-color: light-dark(#c06e2b, #ef9744);
}
.markdown :deep(.markdown-callout-danger),
.markdown :deep(.markdown-callout-failure),
.markdown :deep(.markdown-callout-bug) {
  --callout-color: var(--danger);
}
.markdown :deep(.markdown-callout-abstract),
.markdown :deep(.markdown-callout-tip) {
  --callout-color: light-dark(#046878, #34d3cc);
}
.markdown :deep(.markdown-callout-success) {
  --callout-color: light-dark(#047857, #34d399);
}
.markdown :deep(.markdown-callout-question),
.markdown :deep(.markdown-callout-info) {
  --callout-color: light-dark(#1d4ed8, #60a5fa);
}
.markdown :deep(.markdown-callout-quote) {
  --callout-color: light-dark(#686969, #a2a2a2);
}
.audio {
  padding: 1.5rem 0;
}
.audio audio {
  width: 100%;
}
</style>
