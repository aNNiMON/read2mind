export function itemUrl(path: string): string {
  return `/${path.split("/").map(encodeURIComponent).join("/")}/`;
}

export function itemPath(pathname: string): string {
  return pathname.replace(/^\/+|\/+$/g, "").split("/").map(decodeURIComponent).join("/");
}
