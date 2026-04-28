import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import i18n from "@/i18n";
import type { AppSettings } from "@/lib/types";

const DEFAULT_SETTINGS: AppSettings = {
  server_enabled: false,
  server_host: "127.0.0.1",
  server_port: 8000,
  server_auto_start: false,
  server_log_limit: 100,
  model_path: "",
  auto_copy_result: false,
  save_history: true,
  history_limit: 1000,
  app_language: "en",
  global_shortcut: "command+shift+k",
};

interface SettingsState {
  settings: AppSettings;
  initialized: boolean;
  isUpdating: Record<string, boolean>;

  initialize: () => Promise<void>;
  updateSetting: <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => Promise<void>;
  refreshSettings: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  initialized: false,
  isUpdating: {},

  initialize: async () => {
    if (get().initialized) return;
    try {
      const settings = await invoke<AppSettings>("get_app_settings");
      if (settings.app_language && settings.app_language !== i18n.language) {
        i18n.changeLanguage(settings.app_language);
      }
      set({ settings, initialized: true });
    } catch (e) {
      console.warn("Failed to load settings, using defaults:", e);
      set({ initialized: true });
    }
  },

  updateSetting: async <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => {
    const prev = get().settings[key];
    set((state) => ({
      settings: { ...state.settings, [key]: value },
      isUpdating: { ...state.isUpdating, [key]: true },
    }));
    try {
      await invoke("update_setting", { key, value: JSON.stringify(value) });
      if (key === "app_language") {
        i18n.changeLanguage(value as string);
      }
    } catch (e) {
      set((state) => ({
        settings: { ...state.settings, [key]: prev },
      }));
      console.error(`Failed to update setting ${key}:`, e);
    } finally {
      set((state) => ({
        isUpdating: { ...state.isUpdating, [key]: false },
      }));
    }
  },

  refreshSettings: async () => {
    try {
      const settings = await invoke<AppSettings>("get_app_settings");
      set({ settings });
    } catch (e) {
      console.error("Failed to refresh settings:", e);
    }
  },
}));
