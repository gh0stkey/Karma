import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { appDataDir, appLogDir } from "@tauri-apps/api/path";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { SettingsGroup } from "../ui/SettingsGroup";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { SettingContainer } from "../ui/SettingContainer";
import { Input } from "../ui/Input";
import { Dropdown } from "../ui/Dropdown";
import { PathDisplay } from "../ui/PathDisplay";
import { ShortcutInput } from "./ShortcutInput";
import { useSettings } from "@/hooks/useSettings";

const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "zh-CN", label: "简体中文" },
];

export const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting, isUpdating } = useSettings();
  const [dataDir, setDataDir] = useState("");
  const [logDir, setLogDir] = useState("");

  useEffect(() => {
    appDataDir()
      .then(setDataDir)
      .catch(() => {});
    appLogDir()
      .then(setLogDir)
      .catch(() => {});
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.general.title")}>
        <ShortcutInput descriptionMode="tooltip" grouped={true} />
        <SettingContainer
          title={t("settings.general.language")}
          description={t("settings.general.languageDesc")}
          grouped={true}
        >
          <Dropdown
            options={LANGUAGES}
            selectedValue={settings.app_language}
            onSelect={(v) => updateSetting("app_language", v)}
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("settings.redaction.title")}>
        <ToggleSwitch
          label={t("settings.redaction.autoCopy")}
          description={t("settings.redaction.autoCopyDesc")}
          checked={settings.auto_copy_result}
          onChange={(v) => updateSetting("auto_copy_result", v)}
          isUpdating={isUpdating("auto_copy_result")}
          grouped={true}
        />
        <ToggleSwitch
          label={t("settings.redaction.saveHistory")}
          description={t("settings.redaction.saveHistoryDesc")}
          checked={settings.save_history}
          onChange={(v) => updateSetting("save_history", v)}
          isUpdating={isUpdating("save_history")}
          grouped={true}
        />
        <SettingContainer
          title={t("settings.redaction.historyLimit")}
          description={t("settings.redaction.historyLimitDesc")}
          grouped={true}
        >
          <Input
            variant="compact"
            type="number"
            value={settings.history_limit}
            onChange={(e) =>
              updateSetting("history_limit", parseInt(e.target.value) || 1000)
            }
            className="w-24 text-center"
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("settings.server.title")}>
        <SettingContainer
          title={t("settings.server.logLimit")}
          description={t("settings.server.logLimitDesc")}
          grouped={true}
        >
          <Input
            variant="compact"
            type="number"
            value={settings.server_log_limit}
            onChange={(e) =>
              updateSetting("server_log_limit", parseInt(e.target.value) || 100)
            }
            className="w-24 text-center"
          />
        </SettingContainer>
      </SettingsGroup>

      <SettingsGroup title={t("settings.storage.title")}>
        <SettingContainer
          title={t("settings.storage.dataDir")}
          description={t("settings.storage.dataDirDesc")}
          layout="stacked"
          grouped={true}
        >
          <PathDisplay
            path={dataDir}
            onOpen={() => dataDir && revealItemInDir(dataDir)}
            disabled={!dataDir}
          />
        </SettingContainer>
        <SettingContainer
          title={t("settings.storage.logDir")}
          description={t("settings.storage.logDirDesc")}
          layout="stacked"
          grouped={true}
        >
          <PathDisplay
            path={logDir}
            onOpen={() => logDir && revealItemInDir(logDir)}
            disabled={!logDir}
          />
        </SettingContainer>
      </SettingsGroup>
    </div>
  );
};
