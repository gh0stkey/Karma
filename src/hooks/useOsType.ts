import { type } from "@tauri-apps/plugin-os";
import { type OSType } from "../lib/utils/keyboard";

export function useOsType(): OSType {
  const osType = type();
  if (osType === "macos" || osType === "windows" || osType === "linux") {
    return osType;
  }
  return "unknown";
}
