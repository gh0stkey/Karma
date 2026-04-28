import React, { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelStatus, ServerStatus, LoadedModelInfo } from "@/lib/types";

interface ModelState {
  status: ModelStatus;
  model_path: string;
}

const MODEL_STATUS_DOT: Record<ModelStatus, string> = {
  not_found: "bg-mid-gray",
  ready: "bg-green-500",
  loading: "bg-logo-primary animate-pulse",
  loaded: "bg-green-500",
  error: "bg-red-500",
};

const Footer: React.FC = () => {
  const { t } = useTranslation();
  const [modelState, setModelState] = useState<ModelState>({
    status: "not_found",
    model_path: "",
  });
  const [modelInfo, setModelInfo] = useState<LoadedModelInfo | null>(null);
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);

  const refreshModel = useCallback(async () => {
    try {
      const s = await invoke<ModelState>("get_model_state");
      setModelState(s);
    } catch {}
  }, []);

  const refreshModelInfo = useCallback(async () => {
    try {
      const info = await invoke<LoadedModelInfo>("get_loaded_model_info");
      setModelInfo(info);
    } catch {
      setModelInfo(null);
    }
  }, []);

  const refreshServer = useCallback(async () => {
    try {
      const s = await invoke<ServerStatus>("get_server_status");
      setServerStatus(s);
    } catch {}
  }, []);

  useEffect(() => {
    refreshModel();
    refreshModelInfo();
    refreshServer();
    const interval = setInterval(refreshServer, 5000);
    const unlisten1 = listen<ModelState>("model-state-changed", (e) => {
      setModelState(e.payload);
      if (e.payload.status === "loaded") {
        refreshModelInfo();
      } else {
        setModelInfo(null);
      }
    });
    const unlisten2 = listen<ServerStatus>("server-status-changed", (e) =>
      setServerStatus(e.payload),
    );
    return () => {
      clearInterval(interval);
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
    };
  }, [refreshModel, refreshModelInfo, refreshServer]);

  const modelLabel = modelInfo?.name ?? t("footer.model");

  return (
    <div className="w-full border-t border-mid-gray/20 pt-3">
      <div className="flex justify-between items-center text-xs px-4 pb-3 text-text/60">
        <div className="flex items-center gap-1.5">
          <span
            className={`w-1.5 h-1.5 rounded-full ${MODEL_STATUS_DOT[modelState.status]}`}
          />
          <span>{modelLabel}</span>
          <span className="text-mid-gray">
            {t(`model.status.${modelState.status}`)}
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <span
            className={`w-1.5 h-1.5 rounded-full ${
              serverStatus?.running ? "bg-green-500" : "bg-mid-gray"
            }`}
          />
          <span>
            {serverStatus?.running
              ? `${t("server.status.running")} ${serverStatus.host ?? "127.0.0.1"}:${serverStatus.port ?? 8000}`
              : t("server.status.stopped")}
          </span>
        </div>
      </div>
    </div>
  );
};

export default Footer;
