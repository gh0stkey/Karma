import React, { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Check, Eraser } from "lucide-react";
import { SpanBadge } from "./SpanBadge";
import { Button } from "@/components/ui/Button";
import { Spinner } from "@/components/ui/Spinner";
import { useSettings } from "@/hooks/useSettings";
import { useClipboard } from "@/hooks/useClipboard";
import type { RedactionResult } from "@/lib/types";

export const RedactorPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const [inputText, setInputText] = useState("");
  const [result, setResult] = useState<RedactionResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { copied, copy } = useClipboard();

  const handleRedact = useCallback(async () => {
    if (!inputText.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await invoke<RedactionResult>("redact_text", {
        text: inputText,
      });
      setResult(res);
      if (settings.auto_copy_result) {
        await copy(res.redacted_text);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [inputText, settings.auto_copy_result, copy]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      handleRedact();
    }
  };

  const handleCopy = async () => {
    if (!result) return;
    await copy(result.redacted_text);
  };

  const handleClear = () => {
    setInputText("");
    setResult(null);
    setError(null);
  };

  return (
    <div className="max-w-3xl w-full mx-auto flex flex-col gap-5">
      <div className="space-y-2">
        <div className="flex items-center justify-between px-4">
          <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
            {t("redactor.title")}
          </h2>
          {(inputText || result) && (
            <button
              className="flex items-center gap-1 text-xs text-mid-gray hover:text-text transition-colors cursor-pointer"
              onClick={handleClear}
            >
              <Eraser width={12} height={12} />
              {t("redactor.clearButton")}
            </button>
          )}
        </div>
        <div className="bg-background border border-mid-gray/20 rounded-lg">
          <textarea
            className="w-full px-4 py-3 text-sm bg-transparent rounded-lg resize-y min-h-[80px] focus:outline-none"
            placeholder={t("redactor.inputPlaceholder")}
            value={inputText}
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <hr className="border-mid-gray/20 mx-4" />
          <div className="flex items-center justify-between px-4 py-2.5">
            <Button
              variant="secondary"
              size="sm"
              onClick={handleRedact}
              disabled={!inputText.trim() || loading}
            >
              {loading ? (
                <>
                  <Spinner />
                  <span className="ml-1.5">{t("redactor.redactButton")}</span>
                </>
              ) : (
                t("redactor.redactButton")
              )}
            </Button>
            <span className="text-xs text-mid-gray">
              {t("redactor.shortcut")}
            </span>
          </div>
        </div>
      </div>

      {error && (
        <div className="px-3 py-2 text-xs text-red-500 bg-red-500/10 border border-red-500/20 rounded-lg animate-fade-in-up">
          {error}
        </div>
      )}

      {result && (
        <>
          <div className="space-y-2 animate-fade-in-up">
            <div className="flex items-center justify-between px-4">
              <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
                {t("redactor.outputTitle")}
              </h2>
              <div className="flex items-center gap-3">
                <span className="text-xs text-mid-gray tabular-nums">
                  {result.latency_ms.toFixed(1)}ms
                </span>
                <button
                  className="flex items-center gap-1 text-xs text-mid-gray hover:text-text transition-colors cursor-pointer"
                  onClick={handleCopy}
                >
                  {copied ? (
                    <>
                      <Check width={12} height={12} />
                      {t("redactor.copied")}
                    </>
                  ) : (
                    <>
                      <Copy width={12} height={12} />
                      {t("redactor.copyButton")}
                    </>
                  )}
                </button>
              </div>
            </div>
            <div className="bg-background border border-mid-gray/20 rounded-lg">
              <div className="px-4 py-3 text-sm whitespace-pre-wrap break-words select-text cursor-text leading-relaxed">
                {result.redacted_text}
              </div>
            </div>
          </div>

          {result.detected_spans.length > 0 && (
            <div
              className="space-y-2 animate-fade-in-up"
              style={{ animationDelay: "80ms" }}
            >
              <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide px-4">
                {t("redactor.spansTitle")}
              </h2>
              <div className="bg-background border border-mid-gray/20 rounded-lg">
                {result.detected_spans.map((span, i) => (
                  <React.Fragment key={i}>
                    {i > 0 && <hr className="border-mid-gray/20 mx-4" />}
                    <div className="flex items-center justify-between px-4 py-2.5">
                      <SpanBadge category={span.label} />
                      <span className="text-sm text-text/70" title={span.text}>
                        {span.text}
                      </span>
                    </div>
                  </React.Fragment>
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};
