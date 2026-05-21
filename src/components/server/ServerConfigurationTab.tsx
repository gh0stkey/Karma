import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "lucide-react";
import { SettingContainer } from "../ui/SettingContainer";
import { SettingsGroup } from "../ui/SettingsGroup";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Input } from "../ui/Input";
import type { AppSettings } from "@/lib/types";

interface ServerConfigurationTabProps {
  settings: AppSettings;
  updateSetting: <K extends keyof AppSettings>(
    key: K,
    value: AppSettings[K],
  ) => Promise<void>;
  isUpdating: (key: string) => boolean;
  isServerTransitioning: boolean;
  isServerActive: boolean;
  onServerToggle: (enabled: boolean) => Promise<void>;
}

export const ServerConfigurationTab: React.FC<ServerConfigurationTabProps> = ({
  settings,
  updateSetting,
  isUpdating,
  isServerTransitioning,
  isServerActive,
  onServerToggle,
}) => {
  const { t } = useTranslation();
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);

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
      curl: `curl -X POST ${baseUrl}/redact \\\n+  -H "Content-Type: application/json" \\\n+  -d '{"text": "My name is John and my email is john@example.com"}'`,
    },
  ];

  const toggleEndpoint = (id: string) => {
    setExpandedEndpoint((prev) => (prev === id ? null : id));
  };

  return (
    <>
      <SettingsGroup title={t("server.config")}>
        <ToggleSwitch
          label={t("server.enable")}
          description={t("server.enableDesc")}
          checked={settings.server_enabled}
          onChange={onServerToggle}
          isUpdating={isUpdating("server_enabled") || isServerTransitioning}
          grouped={true}
        />
        <ToggleSwitch
          label={t("server.autoStart")}
          description={t("server.autoStartDesc")}
          checked={settings.server_auto_start}
          onChange={(value) => updateSetting("server_auto_start", value)}
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
        {endpoints.map((endpoint) => (
          <div key={endpoint.id}>
            <div
              className="flex items-center gap-2 px-4 py-2.5 cursor-pointer hover:bg-mid-gray/5 transition-colors"
              onClick={() => toggleEndpoint(endpoint.id)}
            >
              <span className="px-2 py-0.5 rounded text-xs font-mono font-bold shrink-0 bg-mid-gray/15 text-mid-gray">
                {endpoint.method}
              </span>
              <span className="font-mono text-sm text-text/80">{endpoint.path}</span>
              <span className="text-xs text-mid-gray ml-1">
                {endpoint.description}
              </span>
              <ChevronDown
                className={`w-4 h-4 text-mid-gray ml-auto shrink-0 transition-transform ${
                  expandedEndpoint === endpoint.id ? "rotate-180" : ""
                }`}
              />
            </div>
            {expandedEndpoint === endpoint.id && (
              <div className="px-4 pb-3 pt-1">
                <div className="bg-mid-gray/10 rounded-lg p-3 font-mono text-xs text-text/80 whitespace-pre-wrap break-all select-text cursor-text">
                  {endpoint.curl}
                </div>
              </div>
            )}
          </div>
        ))}
      </SettingsGroup>
    </>
  );
};