/** Format an ISO-8601 timestamp as `YYYY-MM-DD HH:MM`, falling back to the raw value. */
export function formatDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;

  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}`
  );
}

export function formatFileSize(size: number): string {
  const values = ["B", "KB", "MB", "GB", "TB"];
  let index = 0;
  while (size >= 1024 && index < values.length - 1) {
    size /= 1024;
    index++;
  }
  return `${size.toFixed(index >= 1 ? 2 : 0)} ${values[index]}`;
}
