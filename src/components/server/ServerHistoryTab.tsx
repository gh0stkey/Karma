import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, Eraser } from "lucide-react";
import type { HttpLogEntry } from "@/lib/types";
import { formatLocalTime } from "@/lib/utils/dateTime";

interface ServerHistoryTabProps {
  httpLogs: HttpLogEntry[];
  onClearLogs: () => Promise<void>;
}

export const ServerHistoryTab: React.FC<ServerHistoryTabProps> = ({
  httpLogs,
  onClearLogs,
}) => {
  const { t } = useTranslation();
  const [expandedLogId, setExpandedLogId] = useState<number | null>(null);

  const toggleLog = (id: number) => {
    setExpandedLogId((prev) => (prev === id ? null : id));
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between px-4">
        <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
          {t("server.httpLog.title")}
        </h2>
        {httpLogs.length > 0 && (
          <button
            className="flex items-center gap-1 text-xs text-mid-gray hover:text-text transition-colors cursor-pointer"
            onClick={onClearLogs}
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
                    {formatLocalTime(log.timestamp)}
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
  );
};