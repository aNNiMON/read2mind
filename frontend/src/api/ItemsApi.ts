import type {
  AiFeature,
  AssetResp,
  CreateItemPayload,
  CreateResp,
  DeleteAttachmentResp,
  DeleteItemResp,
  GenerateAiResp,
  Item,
  ItemQuery,
  ItemResp,
  TagsResp,
  UpdateItemPayload,
  UpdateItemResp,
  UpdateStatusResp,
  UpdateTagsResp,
  UploadResp,
} from "./Item.ts";

const API_URL = import.meta.env.VITE_API_URL ?? "";
const REQUEST_TIMEOUT_MS = 25_000;

/** Encode a path while preserving `/` separators. */
function encodePath(path: string): string {
  return path
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
}

/** Backend expects item paths without a leading `content/` segment. */
function normalizeItemPath(path: string): string {
  const trimmed = path.replace(/^\/+/, "");
  if (trimmed.startsWith("content/")) {
    return trimmed.slice("content/".length);
  }

  // Some API responses return a storage path like `data/2026/<item-path>`.
  // Strip that prefix so attachment writes still target `/api/items/{item_path}/attachments`.
  const dataMatch = trimmed.match(/^data\/\d{4}\/(.+)$/);
  if (dataMatch) {
    return dataMatch[1];
  }

  if (trimmed.startsWith("data/")) {
    return trimmed.slice("data/".length);
  }

  return trimmed;
}

export default class ItemsApi {
  apiCall(
    endpoint: string,
    method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE" = "GET",
    data = {},
  ): Promise<Response> {
    const options: RequestInit = {
      method,
      headers: {
        "Content-Type": "application/json",
      },
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    };

    if (method !== "GET") {
      options.body = JSON.stringify(data);
    }

    return fetch(API_URL + endpoint, options);
  }

  async getItems(query: ItemQuery = {}): Promise<ItemResp> {
    try {
      const tags = [
        ...(query.tags ?? []),
        ...(query.exclude ?? []).map((tag) => `!${tag}`),
      ].filter(Boolean);

      const params = new URLSearchParams();
      if (tags.length) params.set("tags", tags.join(","));
      if (query.kind) params.set("kind", query.kind);
      if (query.status) params.set("status", query.status);
      if (query.date) params.set("date", query.date);
      params.set("limit", String(query.limit ?? 50));
      params.set("offset", String(query.offset ?? 0));

      const endpoint = `/api/items?${params.toString()}`;

      const response = await this.apiCall(endpoint, "GET");

      if (response.ok) {
        const data = (await response.json()) as {
          items: Item[];
          total: number;
        };
        const mapped = data.items.map((item) => ({
          ...item,
          name: item.name ?? item.title ?? "Untitled",
          assets: item.assets ?? [],
        }));
        return {
          ok: true,
          items: mapped,
          total: data.total,
        };
      }

      return {
        ok: false,
        error: `Failed to fetch items. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to fetch items.",
      };
    }
  }

  /** Create an item via the unified items endpoint. */
  async createItem(payload: CreateItemPayload): Promise<CreateResp> {
    try {
      const response = await this.apiCall("/api/items", "POST", payload);

      if (response.ok) {
        const text = await response.text();
        const item = text ? (JSON.parse(text) as Item) : undefined;
        if (item) {
          item.name = item.name ?? item.title ?? "Untitled";
          item.assets = item.assets ?? [];
        }
        return {
          ok: true,
          item,
        };
      }

      return {
        ok: false,
        error: `Failed to create item. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to create item.",
      };
    }
  }

  /**
   * Update an item's status via `PUT /api/items/{item_path}/status`.
   */
  async updateItemStatus(
    path: string,
    status: string,
  ): Promise<UpdateStatusResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}/status`,
        "PUT",
        { status },
      );

      if (response.ok) {
        return { ok: true };
      }

      return {
        ok: false,
        error: `Failed to update status. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to update status.",
      };
    }
  }

  /**
   * Trigger AI generation of a content feature via
   * `PUT /api/items/{item_path}/ai`. Used for `summary.md` / `mindmap.md`.
   */
  async generateAi(path: string, feature: AiFeature): Promise<GenerateAiResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}/ai`,
        "POST",
        { feature },
      );

      if (response.ok) {
        return { ok: true };
      }

      return {
        ok: false,
        error: `Failed to generate ${feature}. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : `Failed to generate ${feature}.`,
      };
    }
  }

  /** Direct URL to a static asset (used for audio playback / iframe fallback). */
  assetUrl(path: string, filename: string): string {
    return `${this.assetBaseUrl(path)}${encodeURIComponent(filename)}`;
  }

  /**
   * Directory URL an item's assets live under (trailing slash included), so
   * relative references inside notes/markdown can be resolved against it.
   */
  assetBaseUrl(path: string): string {
    return `${API_URL}/${encodePath(path)}/`;
  }

  async getAsset(path: string, filename: string): Promise<AssetResp> {
    try {
      const response = await fetch(this.assetUrl(path, filename), {
        cache: "no-store",
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      });

      if (response.ok) {
        return {
          ok: true,
          body: await response.text(),
          contentType: response.headers.get("content-type") ?? "",
        };
      }

      return {
        ok: false,
        status: response.status,
        error: `Failed to fetch content (${response.status}).`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to fetch content.",
      };
    }
  }

  async uploadAttachment(
    path: string,
    file: File,
  ): Promise<UploadResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const form = new FormData();
      form.append("file", file);

      // No explicit Content-Type: the browser sets the multipart boundary.
      const response = await fetch(
        `${API_URL}/api/items/${encodePath(itemPath)}/attachments`,
        { method: "POST", body: form, signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS) },
      );

      if (response.ok) {
        return { ok: true };
      }

      return {
        ok: false,
        error: `Failed to upload file. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to upload file.",
      };
    }
  }

  async updateItemTags(path: string, tags: string): Promise<UpdateTagsResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}/tags`,
        "PUT",
        { tags },
      );

      if (response.ok) {
        return { ok: true };
      }

      return {
        ok: false,
        error: `Failed to update tags. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to update tags.",
      };
    }
  }

  /**
   * Update an item's title, author and created_at via `PUT /api/items/{item_path}`.
   * Since title/created_at drive the item's path, the response item must be used to
   * reopen it afterwards.
   */
  async updateItem(
    path: string,
    payload: UpdateItemPayload,
  ): Promise<UpdateItemResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const body: UpdateItemPayload = {
        title: payload.title,
        created_at: payload.created_at,
      };
      if (payload.author) body.author = payload.author;

      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}`,
        "PUT",
        body,
      );

      if (response.ok) {
        const text = await response.text();
        const item = text ? (JSON.parse(text) as Item) : undefined;
        if (item) {
          item.name = item.name ?? item.title ?? "Untitled";
          item.assets = item.assets ?? [];
        }
        return { ok: true, item };
      }

      return {
        ok: false,
        error: `Failed to update item. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to update item.",
      };
    }
  }

  /** Fetch all known tags with usage counts via `GET /api/tags`. */
  async getTags(): Promise<TagsResp> {
    try {
      const response = await this.apiCall("/api/tags", "GET");

      if (response.ok) {
        const tags = (await response.json()) as Record<string, number>;
        return { ok: true, tags };
      }

      return {
        ok: false,
        error: `Failed to fetch tags. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to fetch tags.",
      };
    }
  }

  async getAttachments(
    path: string,
  ): Promise<
    { ok: true; path: string; attachments: string[] } | {
      ok: false;
      error: string;
    }
  > {
    try {
      const itemPath = normalizeItemPath(path);
      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}/attachments`,
        "GET",
      );

      if (response.ok) {
        const data = (await response.json()) as {
          path: string;
          content: boolean;
          note: boolean;
          attachments: string[];
        };
        const assets = [...data.attachments];
        if (data.content) {
          assets.push("content.md");
        }
        if (data.note) {
          assets.push("note.md");
        }
        return { ok: true, path: data.path, attachments: assets };
      }

      return {
        ok: false,
        error: `Failed to fetch attachments: ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to fetch attachments.",
      };
    }
  }

  /** Delete an item via `DELETE /api/items/{item_path}`. */
  async deleteItem(path: string): Promise<DeleteItemResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}`,
        "DELETE",
      );

      if (response.ok) {
        return { ok: true };
      }

      return {
        ok: false,
        error: `Failed to delete item. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to delete item.",
      };
    }
  }

  /** Delete a single attachment via `DELETE /api/items/{item_path}/attachments/{filename}`. */
  async deleteAttachment(
    path: string,
    filename: string,
  ): Promise<DeleteAttachmentResp> {
    try {
      const itemPath = normalizeItemPath(path);
      const response = await this.apiCall(
        `/api/items/${encodePath(itemPath)}/attachments/${encodeURIComponent(filename)}`,
        "DELETE",
      );

      if (response.ok) {
        return { ok: true };
      }

      return {
        ok: false,
        error: `Failed to delete attachment. ${await response.text()}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : "Failed to delete attachment.",
      };
    }
  }
}
