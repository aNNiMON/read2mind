// Browser history routing requires the window context.
// deno-lint-ignore-file no-window no-window-prefix
import { onScopeDispose, type Ref, ref, watch } from "vue";
import type { Item } from "../api/Item.ts";
import { itemPath, itemUrl } from "../utils/itemRoute.ts";

export default function useItemRoute(
  selected: Ref<Item | null>,
  findItem: (path: string) => Promise<Item | null>,
) {
  const routeLoading = ref(false);
  const routeError = ref<string | null>(null);
  let request = 0;
  let restoring = false;

  const restore = async () => {
    const currentRequest = ++request;
    routeError.value = null;
    routeLoading.value = false;
    restoring = true;
    selected.value = null;
    restoring = false;

    try {
      const path = itemPath(window.location.pathname);
      if (!path) return;
      routeLoading.value = true;
      const item = await findItem(path);
      if (request !== currentRequest) return;
      restoring = true;
      selected.value = item;
      restoring = false;
      if (!item) routeError.value = "Item not found.";
    } catch (error) {
      if (request !== currentRequest) return;
      routeError.value = error instanceof URIError
        ? "Invalid item URL."
        : error instanceof Error
        ? error.message
        : "Failed to open item.";
    } finally {
      if (request === currentRequest) routeLoading.value = false;
    }
  };

  const navigate = (item: Item) => {
    ++request;
    routeLoading.value = false;
    routeError.value = null;
    const url = itemUrl(item.path);
    if (window.location.pathname !== url) window.history.pushState(null, "", url);
    restoring = true;
    selected.value = item;
    restoring = false;
  };

  // Renames and deletions replace the current entry so it stays usable.
  watch(() => selected.value?.path, (path) => {
    if (restoring) return;
    ++request;
    window.history.replaceState(null, "", path ? itemUrl(path) : "/");
  }, { flush: "sync" });

  window.addEventListener("popstate", restore);
  onScopeDispose(() => {
    ++request;
    window.removeEventListener("popstate", restore);
  });
  void restore();

  return { navigate, routeLoading, routeError };
}
