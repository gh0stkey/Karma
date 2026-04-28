export type OSType = "macos" | "windows" | "linux" | "unknown";

export const getKeyName = (
  e: KeyboardEvent,
  osType: OSType = "unknown",
): string => {
  if (e.code) {
    const code = e.code;

    if (code.match(/^F\d+$/)) {
      return code.toLowerCase();
    }

    if (code.match(/^Key[A-Z]$/)) {
      return code.replace("Key", "").toLowerCase();
    }

    if (code.match(/^Digit\d$/)) {
      return code.replace("Digit", "");
    }

    if (code.match(/^Numpad\d$/)) {
      return code.replace("Numpad", "numpad ").toLowerCase();
    }

    const getModifierName = (baseModifier: string): string => {
      switch (baseModifier) {
        case "shift":
          return "shift";
        case "ctrl":
          return "ctrl";
        case "alt":
          return osType === "macos" ? "option" : "alt";
        case "meta":
          if (osType === "macos") return "command";
          return "super";
        default:
          return baseModifier;
      }
    };

    const modifierMap: Record<string, string> = {
      ShiftLeft: getModifierName("shift"),
      ShiftRight: getModifierName("shift"),
      ControlLeft: getModifierName("ctrl"),
      ControlRight: getModifierName("ctrl"),
      AltLeft: getModifierName("alt"),
      AltRight: getModifierName("alt"),
      MetaLeft: getModifierName("meta"),
      MetaRight: getModifierName("meta"),
      OSLeft: getModifierName("meta"),
      OSRight: getModifierName("meta"),
      CapsLock: "caps lock",
      Tab: "tab",
      Enter: "enter",
      Space: "space",
      Backspace: "backspace",
      Delete: "delete",
      Escape: "esc",
      ArrowUp: "up",
      ArrowDown: "down",
      ArrowLeft: "left",
      ArrowRight: "right",
      Home: "home",
      End: "end",
      PageUp: "page up",
      PageDown: "page down",
      Insert: "insert",
      PrintScreen: "print screen",
      ScrollLock: "scroll lock",
      Pause: "pause",
      ContextMenu: "menu",
      NumpadMultiply: "numpad *",
      NumpadAdd: "numpad +",
      NumpadSubtract: "numpad -",
      NumpadDecimal: "numpad .",
      NumpadDivide: "numpad /",
      NumLock: "num lock",
    };

    if (modifierMap[code]) {
      return modifierMap[code];
    }

    const punctuationMap: Record<string, string> = {
      Semicolon: ";",
      Equal: "=",
      Comma: ",",
      Minus: "-",
      Period: ".",
      Slash: "/",
      Backquote: "`",
      BracketLeft: "[",
      Backslash: "\\",
      BracketRight: "]",
      Quote: "'",
    };

    if (punctuationMap[code]) {
      return punctuationMap[code];
    }

    return code.toLowerCase().replace(/([a-z])([A-Z])/g, "$1 $2");
  }

  if (e.key) {
    const key = e.key;

    const keyMap: Record<string, string> = {
      Control: "ctrl",
      Alt: osType === "macos" ? "option" : "alt",
      Shift: "shift",
      Meta:
        osType === "macos" ? "command" : osType === "windows" ? "win" : "super",
      OS:
        osType === "macos" ? "command" : osType === "windows" ? "win" : "super",
      CapsLock: "caps lock",
      ArrowUp: "up",
      ArrowDown: "down",
      ArrowLeft: "left",
      ArrowRight: "right",
      Escape: "esc",
      " ": "space",
    };

    if (keyMap[key]) {
      return keyMap[key];
    }

    return key.toLowerCase();
  }

  return `unknown-${e.keyCode || e.which || 0}`;
};

const capitalizeKey = (key: string): string => {
  if (key === "fn") return "fn";
  if (/^f\d+$/.test(key)) return key.toUpperCase();
  if (key.length === 1) return key.toUpperCase();
  return key.replace(/\b\w/g, (c) => c.toUpperCase());
};

const formatKeyPart = (part: string): string => {
  const trimmed = part.trim();
  if (!trimmed) return "";

  if (trimmed.endsWith("_left")) {
    const name = trimmed.slice(0, -5);
    return `Left ${capitalizeKey(name)}`;
  }
  if (trimmed.endsWith("_right")) {
    const name = trimmed.slice(0, -6);
    return `Right ${capitalizeKey(name)}`;
  }

  return capitalizeKey(trimmed);
};

export const formatKeyCombination = (
  combination: string,
  _osType: OSType,
): string => {
  if (!combination) return "";
  return combination.split("+").map(formatKeyPart).join(" + ");
};

export const normalizeKey = (key: string): string => {
  if (key.startsWith("left ") || key.startsWith("right ")) {
    const parts = key.split(" ");
    if (parts.length === 2) {
      return parts[1];
    }
  }
  return key;
};
