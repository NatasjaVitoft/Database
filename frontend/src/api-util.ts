export function stringToArray(str: string): string[] {
  if (str) {
    return str
      .split(",")
      .map((s) => {
        return s.trim();
      })
      .filter((s) => {
        return s.length > 1;
      });
  }
  return [];
}