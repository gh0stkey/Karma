import React from "react";
import { useTranslation } from "react-i18next";
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
    </>
  );
};