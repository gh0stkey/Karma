import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SectionTabs } from "../ui/SectionTabs";
import { ServerConfigurationTab } from "./ServerConfigurationTab";
import { ServerIntegrationTab } from "./ServerIntegrationTab";
import { useSettings } from "@/hooks/useSettings";
import type { ServerStatus } from "@/lib/types";

type ServerTab = "configuration" | "integration";

export const ServerPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting, isUpdating } = useSettings();
  const [activeTab, setActiveTab] = useState<ServerTab>("configuration");
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);

  const tabs = [
    { id: "configuration", label: t("server.tabs.configuration") },
    { id: "integration", label: t("server.tabs.integration") },
  ] satisfies { id: ServerTab; label: string }[];

  const refreshStatus = useCallback(async () => {
    try {
      const status = await invoke<ServerStatus>("get_server_status");
      setServerStatus(status);
    } catch (e) {
      console.warn("Failed to get server status:", e);
    }
  }, []);

  useEffect(() => {
    refreshStatus();
    const interval = setInterval(refreshStatus, 5000);
    const unlisten = listen<ServerStatus>("server-status-changed", (event) => {
      setServerStatus(event.payload);
    });
    return () => {
      clearInterval(interval);
      unlisten.then((fn) => fn());
    };
  }, [refreshStatus]);

  const serverLifecycleStatus = serverStatus?.status ?? "stopped";
  const isServerTransitioning =
    serverLifecycleStatus === "starting" ||
    serverLifecycleStatus === "stopping";
  const isServerActive =
    serverLifecycleStatus === "running" || isServerTransitioning;

  const handleServerToggle = async (enabled: boolean) => {
    await updateSetting("server_enabled", enabled);
    try {
      if (enabled) {
        await invoke("start_server");
      } else {
        await invoke("stop_server");
      }
    } catch (e) {
      console.warn("Server toggle error:", e);
      refreshStatus();
    }
    refreshStatus();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-3">
      <div className="flex justify-start px-4 pb-2 border-b border-mid-gray/15">
        <SectionTabs
          tabs={tabs}
          activeTab={activeTab}
          onTabChange={setActiveTab}
        />
      </div>

      {activeTab === "configuration" ? (
        <ServerConfigurationTab
          settings={settings}
          updateSetting={updateSetting}
          isUpdating={isUpdating}
          isServerTransitioning={isServerTransitioning}
          isServerActive={isServerActive}
          onServerToggle={handleServerToggle}
        />
      ) : (
        <ServerIntegrationTab settings={settings} />
      )}
    </div>
  );
};
