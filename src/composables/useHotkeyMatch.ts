/** Pure helpers for matching a KeyboardEvent against a hotkey string like "Alt+Shift+S". */

export function matchesHotkey(e: KeyboardEvent, hotkey: string): boolean {
  const parts = hotkey.split("+").map((p) => p.trim().toLowerCase());
  const key = parts[parts.length - 1];
  const needCtrl = parts.includes("ctrl") || parts.includes("control");
  const needAlt = parts.includes("alt");
  const needShift = parts.includes("shift");

  if (e.ctrlKey !== needCtrl || e.altKey !== needAlt || e.shiftKey !== needShift) {
    return false;
  }

  return codeToToken(e.code) === key;
}

/** Convert a KeyboardEvent.code to the lowercase token used in hotkey strings. */
export function codeToToken(code: string): string {
  const letterMatch = code.match(/^Key([A-Z])$/);
  if (letterMatch) return letterMatch[1].toLowerCase();

  const digitMatch = code.match(/^Digit([0-9])$/);
  if (digitMatch) return digitMatch[1];

  const fMatch = code.match(/^F(\d+)$/);
  if (fMatch) return `f${fMatch[1]}`;

  const map: Record<string, string> = {
    Space: "space",
    Tab: "tab",
    Escape: "escape",
    Insert: "insert",
    Delete: "delete",
    Home: "home",
    End: "end",
    PageUp: "pageup",
    PageDown: "pagedown",
    ArrowUp: "up",
    ArrowDown: "down",
    ArrowLeft: "left",
    ArrowRight: "right",
  };
  return map[code] ?? code.toLowerCase();
}
