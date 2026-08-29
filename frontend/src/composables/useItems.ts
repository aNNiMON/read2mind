import { ref, watch } from "vue";
import ItemsApi from "../api/ItemsApi.ts";
import { SearchParser } from "../utils/search.ts";
import type {
  AttachmentMetadata,
  CreateItemPayload,
  CreateResp,
  DeleteAttachmentResp,
  DeleteItemResp,
  Item,
  ItemKind,
  ItemQuery,
  ItemStatus,
  TagFreq,
  UpdateItemPayload,
  UpdateItemResp,
  UpdateStatusResp,
  UpdateTagsResp,
  UploadResp,
} from "../api/Item.ts";

const itemsApi = new ItemsApi();
const searchParser = new SearchParser();

const PAGE_SIZE = 50;

export default function useItems() {
  const items = ref<Item[]>([]);
  const loading = ref(true);
  const loadingMore = ref(false);
  const error = ref<string | null>(null);
  const total = ref(0);
  const hasMore = ref(false);
  const searchQuery = ref<ItemQuery>({});
  const activeKind = ref<ItemKind | null>(null);
  const activeStatus = ref<ItemStatus | null>(null);
  // All known tag names, sorted by descending usage count. Feeds the tags editor's suggestions.
  const allTags = ref<TagFreq[]>([]);

  const loadTags = async () => {
    const response = await itemsApi.getTags();
    if (response.ok) {
      allTags.value = Object.entries(response.tags)
        .sort((a, b) => b[1] - a[1])
        .map(([tag, count]) => ({ name: tag, count }));
    }
  };

  const load = async () => {
    loading.value = true;
    error.value = null;

    const response = await itemsApi.getItems({
      ...searchQuery.value,
      kind: activeKind.value ?? undefined,
      status: activeStatus.value ?? undefined,
      limit: PAGE_SIZE,
      offset: 0,
    });

    loading.value = false;

    if (response.ok === true) {
      items.value = response.items;
      total.value = response.total;
      hasMore.value = items.value.length < response.total;
      return;
    }

    error.value = response.error;
    items.value = [];
    total.value = 0;
    hasMore.value = false;
  };

  const loadMore = async () => {
    if (loading.value || loadingMore.value || !hasMore.value) return;

    loadingMore.value = true;

    const response = await itemsApi.getItems({
      ...searchQuery.value,
      kind: activeKind.value ?? undefined,
      status: activeStatus.value ?? undefined,
      limit: PAGE_SIZE,
      offset: items.value.length,
    });

    loadingMore.value = false;

    if (response.ok === true) {
      items.value = [...items.value, ...response.items];
      total.value = response.total;
      hasMore.value = items.value.length < response.total;
      return;
    }

    error.value = response.error;
  };

  const search = (input: string) => {
    searchQuery.value = searchParser.parseQuery(input);
    return load();
  };

  const setKind = (kind: ItemKind | null) => {
    activeKind.value = kind;
    return load();
  };

  const setStatus = (status: ItemStatus | null) => {
    activeStatus.value = status;
    return load();
  };

  const clearFilters = () => {
    searchQuery.value = {};
    activeKind.value = null;
    activeStatus.value = null;
    return load();
  };

  /**
   * Create an item of any kind. All kinds use the unified `/api/items` endpoint.
   */
  const addItem = async (
    kind: ItemKind,
    payload: CreateItemPayload,
  ): Promise<CreateResp> => {
    const itemPayload: CreateItemPayload = {
      kind,
      url: (kind === "article" || kind === "video") ? payload.url : undefined,
      title: payload.title ?? "",
      content: payload.content,
      tags: payload.tags,
      created_at: payload.created_at,
    };

    const response = await itemsApi.createItem(itemPayload);
    if (response.ok) {
      await load();
    }
    return response;
  };

  const replaceItem = (path: string, update: (item: Item) => Item) => {
    const index = items.value.findIndex((i) => i.path === path);
    if (index === -1) return;

    const updated = update(items.value[index]);
    items.value = [
      ...items.value.slice(0, index),
      updated,
      ...items.value.slice(index + 1),
    ];
    if (selected.value?.path === path) {
      selected.value = updated;
    }
  };

  const updateItemTags = async (
    path: string,
    tags: string,
  ): Promise<UpdateTagsResp> => {
    const response = await itemsApi.updateItemTags(path, tags);

    if (response.ok) {
      const newTags = tags.split(",").map((tag) => tag.trim()).filter(Boolean);
      replaceItem(path, (item) => ({ ...item, tags: newTags }));
    }

    return response;
  };

  const updateItemStatus = async (
    path: string,
    status: ItemStatus,
  ): Promise<UpdateStatusResp> => {
    const response = await itemsApi.updateItemStatus(path, status);

    if (response.ok) {
      replaceItem(path, (item) => ({ ...item, status }));
    }

    return response;
  };

  /**
   * Update an item's title/author/created_at. Title and created_at drive the
   * item's path, so the returned item (with its possibly new path) replaces
   * the old entry and becomes the reopened selection.
   */
  const updateItem = async (
    path: string,
    payload: UpdateItemPayload,
  ): Promise<UpdateItemResp> => {
    const response = await itemsApi.updateItem(path, payload);

    if (response.ok && response.item) {
      const index = items.value.findIndex((i) => i.path === path);
      if (index !== -1) {
        items.value = [
          ...items.value.slice(0, index),
          response.item,
          ...items.value.slice(index + 1),
        ];
      }
      if (selected.value?.path === path) {
        selected.value = response.item;
      }
    }

    return response;
  };

  const deleteItem = async (path: string): Promise<DeleteItemResp> => {
    const response = await itemsApi.deleteItem(path);

    if (response.ok) {
      items.value = items.value.filter((item) => item.path !== path);
      if (selected.value?.path === path) {
        selected.value = null;
      }
    }

    return response;
  };

  const deleteAttachment = async (
    path: string,
    filename: string,
  ): Promise<DeleteAttachmentResp> => {
    const response = await itemsApi.deleteAttachment(path, filename);

    if (response.ok) {
      replaceItem(path, (item) => ({
        ...item,
        assets: item.assets.filter((asset) => asset !== filename),
      }));
    }

    return response;
  };

  const uploadAttachment = async (
    path: string,
    file: File,
  ): Promise<UploadResp> => {
    const response = await itemsApi.uploadAttachment(path, file);

    if (response.ok) {
      replaceItem(
        path,
        (item) =>
          item.assets.includes(file.name)
            ? item
            : { ...item, assets: [...item.assets, file.name] },
      );
    }

    return response;
  };

  const selected = ref<Item | null>(null);

  const select = (item: Item | null) => {
    selected.value = item;
  };

  // Keep the selection in sync with the (possibly reloaded) list; drop it if gone.
  watch(items, (list) => {
    if (!selected.value) return;
    selected.value = list.find((a) => a.path === selected.value?.path) ?? null;
  });

  // Fetch attachments dynamically when selected item changes
  watch(selected, async (newItem) => {
    if (newItem && !newItem.assetPath) {
      const res = await itemsApi.getAttachments(newItem.path);
      if (res.ok) {
        if (res.path && res.path !== newItem.path) {
          newItem.assetPath = res.path;
        }
        newItem.assets = res.attachments;
        newItem.attachmentsMetadata = Object.fromEntries(
          res.metadata.map((meta) => [meta.name, meta]),
        ) as Record<string, AttachmentMetadata>;
      } else {
        newItem.assets = [];
        newItem.attachmentsMetadata = {};
      }
    }
  });

  load();
  loadTags();

  return {
    items,
    error,
    loading,
    loadingMore,
    hasMore,
    total,
    loadMore,
    allTags,
    loadTags,
    activeKind,
    activeStatus,
    selected,
    select,
    reload: load,
    search,
    setKind,
    setStatus,
    clearFilters,
    addItem,
    updateItem,
    updateItemTags,
    updateItemStatus,
    deleteItem,
    deleteAttachment,
    uploadAttachment,
  };
}
