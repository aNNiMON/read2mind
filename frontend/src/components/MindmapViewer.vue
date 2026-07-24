<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from "vue";
import type { IPureNode } from "markmap-common";
// Use the no-plugins entry point to avoid bundling the katex/highlight.js
// plugins markmap-lib enables by default — we only need plain markdown parsing.
import { Transformer } from "markmap-lib/no-plugins";
import { deriveOptions, Markmap } from "markmap-view";
import { ref } from "vue";

const props = defineProps<{ markdown: string }>();

const svgRef = ref<SVGElement>();
let mm: Markmap | null = null;
let savedRoot: IPureNode | null = null;

const transformer = new Transformer();
const markmapOptions = deriveOptions({ maxWidth: 400, colorFreezeLevel: 3 });

async function update(md: string) {
  if (!mm) return;
  const { root } = transformer.transform(md);
  savedRoot = root;
  await mm.setData(root);
  mm.fit();
}

// Set the fold flag on every node. markmap's `initialExpandLevel` only *adds*
// folds and never clears them, so relying on it makes "expand all" a no-op
// after a prior collapse. Setting it explicitly keeps both directions reliable.
function foldAll(node: IPureNode, fold: 0 | 1) {
  node.payload = { ...node.payload, fold };
  node.children?.forEach((child) => foldAll(child, fold));
}

onMounted(() => {
  if (!svgRef.value) return;
  mm = Markmap.create(svgRef.value, markmapOptions);
  update(props.markdown);
});

watch(() => props.markdown, update);

onBeforeUnmount(() => {
  mm?.destroy();
  mm = null;
  savedRoot = null;
});

defineExpose({
  collapseAll: async () => {
    if (!mm || !savedRoot) return;
    foldAll(savedRoot, 1);
    await mm.setData(savedRoot);
    mm.fit();
  },
  expandAll: async () => {
    if (!mm || !savedRoot) return;
    foldAll(savedRoot, 0);
    await mm.setData(savedRoot);
    mm.fit();
  },
  fit: () => mm?.fit(),
});
</script>

<template>
  <div class="mindmap-wrap">
    <svg ref="svgRef" class="mindmap-svg" />
  </div>
</template>

<style scoped>
.mindmap-wrap {
  flex: 1;
  min-height: 0;
  display: flex;
}
.mindmap-svg {
  flex: 1;
  width: 100%;
  height: 100%;
  display: block;
}
/* markmap hardcodes light-mode colors via these vars on the `.markmap` svg;
   map them to the app theme so the mindmap follows light/dark automatically. */
.mindmap-wrap :deep(.markmap) {
  --markmap-text-color: var(--text-h);
  --markmap-a-color: var(--accent);
  --markmap-a-hover-color: var(--accent);
  --markmap-code-bg: var(--code-bg);
  --markmap-code-color: var(--text-h);
  --markmap-circle-open-bg: var(--bg);
}
</style>
