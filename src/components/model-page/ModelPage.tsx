import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "../ui/Button";
import { Spinner } from "../ui/Spinner";
import { SettingsGroup } from "../ui/SettingsGroup";
import { SettingContainer } from "../ui/SettingContainer";
import type { LoadedModelInfo } from "@/lib/types";

interface ModelState {
  status: string;
  model_path: string;
}

export const ModelPage: React.FC = () => {
  const { t } = useTranslation();
  const [state, setState] = useState<ModelState>({
    status: "not_found",
    model_path: "",
  });
  const [pathInput, setPathInput] = useState("");
  const [modelInfo, setModelInfo] = useState<LoadedModelInfo | null>(null);

  const refreshState = useCallback(async () => {
    try {
      const s = await invoke<ModelState>("get_model_state");
      setState(s);
      if (s.model_path) setPathInput(s.model_path);
    } catch (e) {
      console.warn("Failed to get model state:", e);
    }
  }, []);

  const refreshModelInfo = useCallback(async () => {
    try {
      const info = await invoke<LoadedModelInfo>("get_loaded_model_info");
      setModelInfo(info);
    } catch {
      setModelInfo(null);
    }
  }, []);

  useEffect(() => {
    refreshState();
    refreshModelInfo();
    const unlisten = listen<ModelState>("model-state-changed", (event) => {
      setState(event.payload);
      if (event.payload.status === "loaded") {
        refreshModelInfo();
      } else if (
        event.payload.status === "not_found" ||
        event.payload.status === "error"
      ) {
        setModelInfo(null);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refreshState, refreshModelInfo]);

  const handleBrowse = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true });
      if (selected) {
        setPathInput(selected as string);
        await invoke("set_model_path", { path: selected });
        await invoke("reload_model");
        refreshState();
      }
    } catch (e) {
      console.error("Browse failed:", e);
    }
  };

  const isLoaded = state.status === "loaded";
  const isLoading = state.status === "loading";

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("model.config.title")}>
        <SettingContainer
          title={t("model.config.path")}
          description={t("model.config.pathDesc")}
          layout="stacked"
          grouped={true}
        >
          <div className="flex items-center gap-2">
            <div className="flex-1 min-w-0 px-2 py-1 bg-mid-gray/10 border border-mid-gray/80 rounded-md text-sm font-mono break-all select-text cursor-text">
              {pathInput || t("model.config.pathPlaceholder")}
            </div>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleBrowse}
              disabled={isLoading}
            >
              {isLoading ? (
                <>
                  <Spinner />
                  <span className="ml-1.5">{t("model.status.loading")}</span>
                </>
              ) : (
                t("model.config.browse")
              )}
            </Button>
          </div>
        </SettingContainer>
      </SettingsGroup>

      {isLoaded && modelInfo && (
        <SettingsGroup title={t("model.info.title")}>
          <SettingContainer
            title={t("model.info.name")}
            description=""
            descriptionMode="inline"
            grouped={true}
          >
            <span className="text-sm font-mono text-text/80">
              {modelInfo.name}
            </span>
          </SettingContainer>
          <SettingContainer
            title={t("model.info.architecture")}
            description=""
            descriptionMode="inline"
            grouped={true}
          >
            <span className="text-sm font-mono text-text/80">
              {modelInfo.architecture}
            </span>
          </SettingContainer>
          <SettingContainer
            title={t("model.info.hiddenSize")}
            description=""
            descriptionMode="inline"
            grouped={true}
          >
            <span className="text-sm font-mono text-text/80">
              {modelInfo.hidden_size}
            </span>
          </SettingContainer>
          <SettingContainer
            title={t("model.info.vocabSize")}
            description=""
            descriptionMode="inline"
            grouped={true}
          >
            <span className="text-sm font-mono text-text/80">
              {modelInfo.vocab_size.toLocaleString()}
            </span>
          </SettingContainer>
          <SettingContainer
            title={t("model.info.maxPosition")}
            description=""
            descriptionMode="inline"
            grouped={true}
          >
            <span className="text-sm font-mono text-text/80">
              {modelInfo.max_position_embeddings.toLocaleString()}
            </span>
          </SettingContainer>
          <SettingContainer
            title={t("model.info.numLabels")}
            description=""
            descriptionMode="inline"
            grouped={true}
          >
            <span className="text-sm font-mono text-text/80">
              {modelInfo.num_labels}
            </span>
          </SettingContainer>
        </SettingsGroup>
      )}
    </div>
  );
};
