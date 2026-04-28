import { useEffect } from "react";
import { useSettingsStore } from "@/stores/settingsStore";

export function useSettings() {
  const store = useSettingsStore();

  useEffect(() => {
    store.initialize();
  }, []);

  return {
    settings: store.settings,
    initialized: store.initialized,
    updateSetting: store.updateSetting,
    isUpdating: (key: string) => store.isUpdating[key] ?? false,
  };
}
