export function truncateText(value: string, length = 120) {
  return value.length > length ? `${value.slice(0, length).trim()}...` : value;
}
