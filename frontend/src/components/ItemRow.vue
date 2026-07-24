<script setup lang="ts">
import { computed } from "vue";
import type { Item } from "../api/Item";
import { kindMeta, statusMeta } from "../api/meta";
import { formatDate } from "../utils/format";

const props = defineProps<{
  item: Item;
  active: boolean;
}>();

const kind = computed(() => kindMeta(props.item.kind));
const status = computed(() => statusMeta(props.item.status ?? "new"));
</script>

<template>
  <button class="list-item" :class="{ active }" type="button">
    <div class="list-item-meta">
      <span class="list-item-time">
        <span
          v-if="kind"
          class="kind-dot"
          :style="{ background: kind.color }"
          :title="kind.label"
        ></span>
        <span class="list-item-date">{{ formatDate(item.created_at) }}</span>
      </span>
      <span
        v-if="status"
        class="status-badge"
        :style="{ color: status.color }"
      >
        {{ status.label }}
      </span>
    </div>
    <div class="list-item-title">{{ item.name }}</div>
    <div v-if="item.tags.length" class="list-item-tags">
      <span v-for="tag in item.tags" :key="tag" class="tag">{{ tag }}</span>
    </div>
  </button>
</template>

<style scoped>
.list-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: 0.875rem 1rem;
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  background: var(--bg);
  color: var(--text);
  cursor: pointer;
  font: inherit;
  transition: border-color 0.2s, box-shadow 0.2s;
}
.list-item:hover {
  border-color: var(--accent-border);
}
.list-item.active {
  border-color: var(--accent);
  box-shadow: var(--shadow);
}
.list-item-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  font-size: 0.8em;
  color: var(--text);
  margin-bottom: 0.25rem;
}
.list-item-time {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}
.status-badge {
  flex-shrink: 0;
  font-size: 0.625rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  line-height: 1;
  padding: 0.4em 0.8em;
  border-radius: 999px;
  background: color-mix(in srgb, currentColor 14%, transparent);
  pointer-events: none;
}
.list-item-date {
  opacity: 0.7;
}
.kind-dot {
  width: 0.5625rem;
  height: 0.5625rem;
  border-radius: 50%;
  flex-shrink: 0;
}
.list-item-title {
  font-family: var(--heading);
  font-size: 1rem;
  color: var(--text-h);
  line-height: 125%;
}
.list-item-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.1rem;
  margin-top: 0.5rem;
}
.tag {
  font-size: 0.7em;
  padding: 0 0.3rem;
  border-radius: 0.6rem;
  color: var(--accent);
  background: var(--accent-bg);
}
</style>
