export type ItemKind = "article" | "note" | "task" | "video";
export type ItemStatus =
  | "new"
  | "planned_high"
  | "planned"
  | "planned_low"
  | "in_progress"
  | "paused"
  | "done"
  | "rejected";

export interface Item {
  path: string;
  assetPath?: string;
  url?: string;
  title?: string;
  author?: string;
  name: string;
  assets: string[];
  attachmentsMetadata: Record<string, AttachmentMetadata>;
  tags: string[];
  created_at: string;
  kind?: ItemKind;
  status?: ItemStatus;
}

export type ItemResp =
  | { ok: true; items: Item[]; total: number }
  | { ok: false; error: string };

export type CreateResp =
  | { ok: true; item?: Item }
  | { ok: false; error: string };

export interface CreateItemPayload {
  kind: ItemKind;
  title?: string;
  url?: string;
  tags?: string;
  content?: string;
  created_at?: string;
}

export type AssetResp =
  | { ok: true; body: string; contentType: string }
  | { ok: false; status?: number; error: string };

export interface AttachmentMetadata {
  name: string;
  size: number;
}

export type Attachment = {
  path: string;
  content: boolean;
  note: boolean;
  attachments: string[];
  metadata: AttachmentMetadata[];
};

export type AttachmentsResp =
  | { ok: true; path: string; attachments: string[]; metadata: AttachmentMetadata[] }
  | { ok: false; error: string };

export type UploadResp = { ok: true } | { ok: false; error: string };

export type UpdateTagsResp = { ok: true } | { ok: false; error: string };

/** All known tags with their usage counts, from `GET /api/tags`. */
export type TagsResp =
  | { ok: true; tags: Record<string, number> }
  | { ok: false; error: string };

export type TagFreq = { name: string; count: number };

export type UpdateStatusResp = { ok: true } | { ok: false; error: string };

export interface UpdateItemPayload {
  title: string;
  created_at: string;
  author?: string;
}

export type UpdateItemResp =
  | { ok: true; item?: Item }
  | { ok: false; error: string };

export type DeleteItemResp = { ok: true } | { ok: false; error: string };

export type DeleteAttachmentResp = { ok: true } | { ok: false; error: string };

/** AI-generatable content features, driving `PUT /api/items/{item_path}/ai`. */
export type AiFeature = "summary" | "mindmap";

export type GenerateAiResp = { ok: true } | { ok: false; error: string };

/** Query for tag-based filtering: included tags are AND-ed, excluded tags are `!`-prefixed. */
export interface ItemQuery {
  tags?: string[];
  exclude?: string[];
  kind?: ItemKind;
  status?: ItemStatus;
  /** Single-day filter in `YYYY-MM-DD` format. */
  date?: string;
  /** Max number of items to return. */
  limit?: number;
  /** Number of items to skip, for pagination. */
  offset?: number;
}
