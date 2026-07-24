import type { AiFeature, ItemKind } from "./Item.ts";

export type ContentKind = "html" | "markdown" | "audio" | "mindmap";

export interface ContentTab {
  key: string;
  label: string;
  filename: string;
  kind: ContentKind;
  /** When set, the asset is produced by AI generation rather than upload. */
  aiFeature?: AiFeature;
  /** When set, the raw asset can be edited in place (see ContentViewer). */
  editable?: boolean;
}

/** The full catalog of content tabs, keyed for reuse across item kinds. */
const TAB = {
  html: { key: "html", label: "HTML", filename: "content.html", kind: "html" },
  markdown: {
    key: "markdown",
    label: "Markdown",
    filename: "content.md",
    kind: "markdown",
    editable: true,
  },
  transcript: {
    key: "transcript",
    label: "Transcript",
    filename: "content.md",
    kind: "markdown",
    editable: true,
  },
  summary: {
    key: "summary",
    label: "Summary",
    filename: "summary.md",
    kind: "markdown",
    aiFeature: "summary",
  },
  mindmap: {
    key: "mindmap",
    label: "MindMap",
    filename: "mindmap.md",
    kind: "mindmap",
    aiFeature: "mindmap",
  },
  audio: { key: "audio", label: "Audio", filename: "audio.mp3", kind: "audio" },
  note: { key: "note", label: "Note", filename: "note.md", kind: "markdown" },
} as const satisfies Record<string, ContentTab>;

/** Default fallback tab list (articles), used when a kind is unknown. */
export const CONTENT_TABS: ContentTab[] = [
  TAB.html,
  TAB.markdown,
  TAB.summary,
  TAB.mindmap,
  TAB.audio,
];

const TABS_BY_KIND: Record<ItemKind, ContentTab[]> = {
  // The trailing `note` tab renders the user's own `note.md` (see NoteEditor).
  // note/task kinds are their own note, so they get no separate Note tab.
  article: [TAB.markdown, TAB.summary, TAB.mindmap, TAB.audio, TAB.note],
  note: [TAB.markdown],
  task: [TAB.markdown],
  video: [TAB.transcript, TAB.summary, TAB.mindmap, TAB.audio, TAB.note],
};

/** Resolve the content tabs available for a given item kind. */
export function tabsForKind(kind: ItemKind | undefined): ContentTab[] {
  return kind ? TABS_BY_KIND[kind] : CONTENT_TABS;
}

/** Whether an item kind supports a separate user note (`note.md`). */
export function kindHasNote(kind: ItemKind | undefined): boolean {
  return kind !== "note" && kind !== "task";
}
