export function sameAnswerSet(left: string[], right: string[]) {
  const normalize = (value: string) => value.trim().toUpperCase();
  return [...new Set(left.map(normalize))].sort().join("|") === [...new Set(right.map(normalize))].sort().join("|");
}
