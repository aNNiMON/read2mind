import type { ItemKind, ItemStatus } from "./Item.ts";

export interface KindMeta {
  value: ItemKind;
  label: string;
  color: string;
}

export interface StatusMeta {
  value: ItemStatus;
  label: string;
  color: string;
}

/** Color-coded catalog of item kinds, reused across selectors, menus and lists. */
export const KINDS: KindMeta[] = [
  { value: "article", label: "Article", color: "#aa3bff" },
  { value: "note", label: "Note", color: "#eab308" },
  { value: "task", label: "Task", color: "#22c55e" },
  { value: "video", label: "Video", color: "#ef4444" },
];

/** Color-coded catalog of item statuses. `new` is the implicit default. */
export const STATUSES: StatusMeta[] = [
  { value: "new", label: "New", color: "#8b5cf6" },
  { value: "planned_high", label: "Planned High", color: "#d96906" },
  { value: "planned", label: "Planned", color: "#d98f17" },
  { value: "planned_low", label: "Planned Low", color: "#d0b167" },
  { value: "in_progress", label: "In progress", color: "#3b82f6" },
  { value: "paused", label: "Paused", color: "#6b7280" },
  { value: "done", label: "Done", color: "#2e8b57" },
  { value: "rejected", label: "Rejected", color: "#ef4444" },
];

const KIND_BY_VALUE = new Map(KINDS.map((k) => [k.value, k]));
const STATUS_BY_VALUE = new Map(STATUSES.map((s) => [s.value, s]));

export function kindMeta(
  kind: ItemKind | undefined | null,
): KindMeta | undefined {
  return kind ? KIND_BY_VALUE.get(kind) : undefined;
}

export function statusMeta(
  status: ItemStatus | undefined | null,
): StatusMeta | undefined {
  return status ? STATUS_BY_VALUE.get(status) : undefined;
}
