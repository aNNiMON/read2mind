import { type Ref, ref, watch } from "vue";
import ItemsApi from "../api/ItemsApi.ts";
import type { Item } from "../api/Item.ts";
import type { ContentTab } from "../api/contentTypes.ts";

const itemsApi = new ItemsApi();

/**
 * Fetches the asset backing the currently selected tab of the selected
 * item. `missing` is true when the asset doesn't exist yet (404), which
 * the viewer uses to render the upload area instead of content.
 */
export default function useItemContent(
  item: Ref<Item | null>,
  tab: Ref<ContentTab>,
) {
  const body = ref("");
  const contentType = ref("");
  const loading = ref(false);
  const missing = ref(false);
  const error = ref<string | null>(null);

  // Guards against races between overlapping loads (e.g. rapid tab
  // switching): only the most recently issued request is allowed to write
  // its result into the refs above.
  let requestId = 0;

  // `silent` is used for background refreshes (e.g. after a note save) so the
  // already-rendered content stays mounted and its scroll position survives,
  // instead of being torn down behind a "Loading…" placeholder.
  const load = async (opts?: { silent?: boolean }) => {
    const silent = !!opts?.silent;
    const thisRequestId = ++requestId;
    const current = item.value;
    const itemPath = current?.assetPath;
    if (!current || !itemPath) {
      body.value = "";
      missing.value = false;
      error.value = null;
      return;
    }

    if (!current.assets.includes(tab.value.filename)) {
      loading.value = false;
      missing.value = true;
      error.value = null;
      body.value = "";
      return;
    }

    if (!silent) {
      loading.value = true;
      body.value = "";
    }
    error.value = null;
    missing.value = false;

    const response = await itemsApi.getAsset(itemPath, tab.value.filename);

    if (thisRequestId !== requestId) {
      // A newer load has started since this one was issued; drop this result.
      return;
    }

    loading.value = false;

    if (response.ok === true) {
      body.value = response.body;
      contentType.value = response.contentType;
      return;
    }

    if (response.status === 404) {
      missing.value = true;
      return;
    }

    error.value = response.error;
  };

  const assetUrl = () =>
    item.value?.assetPath ? itemsApi.assetUrl(item.value.assetPath, tab.value.filename) : "";

  watch([() => item.value?.assetPath ?? item.value?.path, tab], () => load(), {
    immediate: true,
  });

  return {
    body,
    contentType,
    loading,
    missing,
    error,
    assetUrl,
    reload: () => load({ silent: true }),
  };
}
