import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { appLogDir, join } from "@tauri-apps/api/path";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { ChevronDown } from "lucide-react";
import { SettingContainer } from "../ui/SettingContainer";
import { SettingsGroup } from "../ui/SettingsGroup";
import { PathDisplay } from "../ui/PathDisplay";
import type { AppSettings } from "@/lib/types";

interface ServerIntegrationTabProps {
  settings: AppSettings;
}

export const ServerIntegrationTab: React.FC<ServerIntegrationTabProps> = ({
  settings,
}) => {
  const { t } = useTranslation();
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);
  const [logPath, setLogPath] = useState("");

  useEffect(() => {
    appLogDir()
      .then((dir) => join(dir, "http.log"))
      .then(setLogPath)
      .catch(() => {});
  }, []);

  const baseUrl = `http://${settings.server_host}:${settings.server_port}`;
  const endpoints = [
    {
      id: "health",
      method: "GET",
      path: "/health",
      description: t("server.apiRef.health"),
      curl: `curl ${baseUrl}/health`,
      response: `{
  "status": "ok",
  "model_loaded": true
}`,
    },
    {
      id: "redact",
      method: "POST",
      path: "/redact",
      description: t("server.apiRef.redact"),
      curl: `curl -X POST ${baseUrl}/redact \
  -H "Content-Type: application/json" \
  -d '{"text": "My name is John and my email is john@example.com"}'`,
      response: `{
  "schema_version": 1,
  "text": "My name is John and my email is john@example.com",
  "redacted_text": "My name is [PERSON] and my email is [EMAIL]",
  "detected_spans": [
    { "label": "private_person", "start": 11, "end": 15, "text": "John", "placeholder": "[PERSON]" },
    { "label": "private_email", "start": 32, "end": 48, "text": "john@example.com", "placeholder": "[EMAIL]" }
  ],
  "summary": { "private_person": 1, "private_email": 1 },
  "latency_ms": 12.3
}`,
    },
  ];

  const toggleEndpoint = (id: string) => {
    setExpandedEndpoint((prev) => (prev === id ? null : id));
  };

  return (
    <>
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
              <span className="font-mono text-sm text-text/80">
                {endpoint.path}
              </span>
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
              <div className="px-4 pb-3 pt-1 space-y-2">
                <div className="bg-mid-gray/10 rounded-lg p-3 font-mono text-xs text-text/80 whitespace-pre-wrap break-all select-text cursor-text">
                  {endpoint.curl}
                </div>
                <p className="text-xs text-text/50">
                  {t("server.apiRef.response")}
                </p>
                <div className="bg-mid-gray/10 rounded-lg p-3 font-mono text-xs text-text/80 whitespace-pre-wrap break-all select-text cursor-text">
                  {endpoint.response}
                </div>
              </div>
            )}
          </div>
        ))}
      </SettingsGroup>

      <SettingsGroup title={t("server.integration.logTitle")}>
        <SettingContainer
          title={t("server.integration.logFile")}
          description={t("server.integration.logDesc")}
          layout="stacked"
          grouped={true}
        >
          <PathDisplay
            path={logPath}
            onOpen={() => logPath && revealItemInDir(logPath)}
            disabled={!logPath}
          />
        </SettingContainer>
      </SettingsGroup>
    </>
  );
};
