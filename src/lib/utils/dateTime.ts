const localDateTimeFormatter = new Intl.DateTimeFormat(undefined, {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

const localTimeFormatter = new Intl.DateTimeFormat(undefined, {
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
});

export function parseAppTimestamp(timestamp: string) {
  const direct = new Date(timestamp);
  if (!Number.isNaN(direct.getTime())) {
    return direct;
  }

  const sqliteUtc = new Date(timestamp.replace(" ", "T") + "Z");
  if (!Number.isNaN(sqliteUtc.getTime())) {
    return sqliteUtc;
  }

  return null;
}

export function formatLocalDateTime(timestamp: string) {
  const parsed = parseAppTimestamp(timestamp);
  return parsed ? localDateTimeFormatter.format(parsed) : timestamp;
}

export function formatLocalTime(timestamp: string) {
  const parsed = parseAppTimestamp(timestamp);
  return parsed ? localTimeFormatter.format(parsed) : timestamp;
}