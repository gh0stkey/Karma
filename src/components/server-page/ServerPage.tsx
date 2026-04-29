import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ChevronDown, Eraser } from "lucide-react";
import { SettingsGroup } from "../ui/SettingsGroup";
import { SettingContainer } from "../ui/SettingContainer";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Input } from "../ui/Input";
import { useSettings } from "@/hooks/useSettings";
import type { ServerStatus, HttpLogEntry } from "@/lib/types";

export const ServerPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting, isUpdating } = useSettings();
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);
  const [httpLogs, setHttpLogs] = useState<HttpLogEntry[]>([]);
  const [expandedLogId, setExpandedLogId] = useState<number | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const status = await invoke<ServerStatus>("get_server_status");
      setServerStatus(status);
    } catch (e) {
      console.warn("Failed to get server status:", e);
    }
  }, []);

  const refreshLogs = useCallback(async () => {
    try {
      const logs = await invoke<HttpLogEntry[]>("get_http_logs", {
        limit: settings.server_log_limit,
      });
      setHttpLogs(logs);
    } catch {
      // Backend command may not exist yet
    }
  }, [settings.server_log_limit]);

  useEffect(() => {
    refreshStatus();
    refreshLogs();
    const interval = setInterval(refreshStatus, 5000);
    const logInterval = setInterval(refreshLogs, 3000);
    const unlisten = listen<ServerStatus>("server-status-changed", (event) => {
      setServerStatus(event.payload);
    });
    const unlistenLog = listen<HttpLogEntry>("http-log-entry", (event) => {
      setHttpLogs((prev) =>
        [event.payload, ...prev].slice(0, settings.server_log_limit),
      );
    });
    return () => {
      clearInterval(interval);
      clearInterval(logInterval);
      unlisten.then((fn) => fn());
      unlistenLog.then((fn) => fn());
    };
  }, [refreshStatus, refreshLogs, settings.server_log_limit]);

  const baseUrl = `http://${settings.server_host}:${settings.server_port}`;

  const endpoints = [
    {
      id: "health",
      method: "GET",
      path: "/health",
      description: t("server.apiRef.health"),
      curl: `curl ${baseUrl}/health`,
    },
    {
      id: "redact",
      method: "POST",
      path: "/redact",
      description: t("server.apiRef.redact"),
      curl: `curl -X POST ${baseUrl}/redact \\\n  -H "Content-Type: application/json" \\\n  -d '{"text": "My name is John and my email is john@example.com"}'`,
    },
  ];

  const toggleEndpoint = (id: string) => {
    setExpandedEndpoint((prev) => (prev === id ? null : id));
  };

  const toggleLog = (id: number) => {
    setExpandedLogId((prev) => (prev === id ? null : id));
  };

  const handleClearLogs = async () => {
    await invoke("clear_http_logs").catch(() => {});
    setHttpLogs([]);
  };

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
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("server.config")}>
        <ToggleSwitch
          label={t("server.enable")}
          description={t("server.enableDesc")}
          checked={settings.server_enabled}
          onChange={handleServerToggle}
          isUpdating={isUpdating("server_enabled") || isServerTransitioning}
          grouped={true}
        />
        <ToggleSwitch
          label={t("server.autoStart")}
          description={t("server.autoStartDesc")}
          checked={settings.server_auto_start}
          onChange={(v) => updateSetting("server_auto_start", v)}
          isUpdating={isUpdating("server_auto_start")}
          grouped={true}
          disabled={!settings.server_enabled}
        />
        <SettingContainer
          title={t("server.host")}
          description={t("server.hostDesc")}
          grouped={true}
        >
          <Input
            variant="compact"
            value={settings.server_host}
            onChange={(e) => updateSetting("server_host", e.target.value)}
            className="w-36 text-center"
            disabled={isServerActive}
          />
        </SettingContainer>
        <SettingContainer
          title={t("server.port")}
          description={t("server.portDesc")}
          grouped={true}
        >
          <Input
            variant="compact"
            type="number"
            value={settings.server_port}
            onChange={(e) =>
              updateSetting("server_port", parseInt(e.target.value) || 8000)
            }
            className="w-24 text-center"
            disabled={isServerActive}
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("server.apiRef.title")}>
        {endpoints.map((ep) => (
          <div key={ep.id}>
            <div
              className="flex items-center gap-2 px-4 py-2.5 cursor-pointer hover:bg-mid-gray/5 transition-colors"
              onClick={() => toggleEndpoint(ep.id)}
            >
              <span className="px-2 py-0.5 rounded text-xs font-mono font-bold shrink-0 bg-mid-gray/15 text-mid-gray">
                {ep.method}
              </span>
              <span className="font-mono text-sm text-text/80">{ep.path}</span>
              <span className="text-xs text-mid-gray ml-1">
                {ep.description}
              </span>
              <ChevronDown
                className={`w-4 h-4 text-mid-gray ml-auto shrink-0 transition-transform ${
                  expandedEndpoint === ep.id ? "rotate-180" : ""
                }`}
              />
            </div>
            {expandedEndpoint === ep.id && (
              <div className="px-4 pb-3 pt-1">
                <div className="bg-mid-gray/10 rounded-lg p-3 font-mono text-xs text-text/80 whitespace-pre-wrap break-all select-text cursor-text">
                  {ep.curl}
                </div>
              </div>
            )}
          </div>
        ))}
      </SettingsGroup>

      <div className="space-y-2">
        <div className="flex items-center justify-between px-4">
          <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
            {t("server.httpLog.title")}
          </h2>
          {httpLogs.length > 0 && (
            <button
              className="flex items-center gap-1 text-xs text-mid-gray hover:text-text transition-colors cursor-pointer"
              onClick={handleClearLogs}
            >
              <Eraser width={12} height={12} />
              {t("server.httpLog.clear")}
            </button>
          )}
        </div>
        <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
          {httpLogs.length === 0 ? (
            <div className="px-4 py-6 text-center text-sm text-mid-gray">
              {t("server.httpLog.empty")}
            </div>
          ) : (
            httpLogs.map((log, i) => (
              <React.Fragment key={log.id}>
                {i > 0 && <hr className="border-mid-gray/20 mx-4" />}
                <div>
                  <div
                    className="flex items-center gap-2 px-4 py-2.5 cursor-pointer hover:bg-mid-gray/5 transition-colors"
                    onClick={() => toggleLog(log.id)}
                  >
                    <span className="px-2 py-0.5 rounded text-xs font-mono font-bold shrink-0 bg-mid-gray/15 text-mid-gray">
                      {log.method}
                    </span>
                    <span className="font-mono text-sm text-text/80 truncate">
                      {log.path}
                    </span>
                    <span className="px-1.5 py-0.5 rounded text-xs font-mono font-bold shrink-0 bg-mid-gray/15 text-mid-gray">
                      {log.status}
                    </span>
                    <span className="text-xs text-mid-gray tabular-nums shrink-0">
                      {log.latency_ms}ms
                    </span>
                    <span className="text-xs text-mid-gray ml-auto shrink-0">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </span>
                    <ChevronDown
                      className={`w-4 h-4 text-mid-gray shrink-0 transition-transform ${
                        expandedLogId === log.id ? "rotate-180" : ""
                      }`}
                    />
                  </div>
                  {expandedLogId === log.id && (
                    <div className="px-4 pb-3 space-y-2">
                      {log.request_body && (
                        <div>
                          <p className="text-xs text-text/50 mb-1">
                            {t("server.httpLog.request")}
                          </p>
                          <div className="bg-mid-gray/10 rounded-lg p-3 font-mono text-xs text-text/80 whitespace-pre-wrap break-all select-text cursor-text">
                            {log.request_body}
                          </div>
                        </div>
                      )}
                      {log.response_body && (
                        <div>
                          <p className="text-xs text-text/50 mb-1">
                            {t("server.httpLog.response")}
                          </p>
                          <div className="bg-mid-gray/10 rounded-lg p-3 font-mono text-xs text-text/80 whitespace-pre-wrap break-all select-text cursor-text">
                            {log.response_body}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </React.Fragment>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
