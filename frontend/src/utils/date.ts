const DATE_ONLY_RE = /^(\d{4})-(\d{2})-(\d{2})$/;
const DATETIME_RE = /^(\d{4})-(\d{2})-(\d{2})\s(\d{2}):(\d{2}):(\d{2})$/;

function toApiDateTime(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  const offsetMinutes = -date.getTimezoneOffset();
  const sign = offsetMinutes >= 0 ? "+" : "-";
  const absOffset = Math.abs(offsetMinutes);
  const offsetHours = Math.floor(absOffset / 60);
  const offsetMins = absOffset % 60;

  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}` +
    `${sign}${pad(offsetHours)}:${pad(offsetMins)}`
  );
}

function buildDate(
  year: number,
  month: number,
  day: number,
  hour: number,
  minute: number,
  second: number,
): Date | null {
  const date = new Date(year, month - 1, day, hour, minute, second, 0);

  if (
    date.getFullYear() !== year ||
    date.getMonth() !== month - 1 ||
    date.getDate() !== day ||
    date.getHours() !== hour ||
    date.getMinutes() !== minute ||
    date.getSeconds() !== second
  ) {
    return null;
  }

  return date;
}

/**
 * Parse user-friendly date input into API timestamp format.
 * Accepts only `YYYY-MM-DD` or `YYYY-MM-DD HH:mm:ss`.
 */
export function parseCreatedAtInput(input: string): string | null {
  const dateOnlyMatch = input.match(DATE_ONLY_RE);
  if (dateOnlyMatch) {
    const [, y, m, d] = dateOnlyMatch;
    const date = buildDate(Number(y), Number(m), Number(d), 0, 0, 0);
    return date ? toApiDateTime(date) : null;
  }

  const dateTimeMatch = input.match(DATETIME_RE);
  if (dateTimeMatch) {
    const [, y, m, d, hh, mm, ss] = dateTimeMatch;
    const date = buildDate(
      Number(y),
      Number(m),
      Number(d),
      Number(hh),
      Number(mm),
      Number(ss),
    );
    return date ? toApiDateTime(date) : null;
  }

  return null;
}
