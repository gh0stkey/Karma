import React, { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Check, Copy, Pin, PinOff } from "lucide-react";
import { SpanBadge } from "../redactor/SpanBadge";
import { Spinner } from "@/components/ui/Spinner";
import { useSettings } from "@/hooks/useSettings";
import type { RedactionResult } from "@/lib/types";

const DEBOUNCE_MS = 400;

export const QuickRedactPage: React.FC = () => {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const [inputText, setInputText] = useState("");
  const [result, setResult] = useState<RedactionResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pinned, setPinned] = useState(false);
  const [copied, setCopied] = useState(false);
  const copyTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const requestSeq = useRef(0);

  const copy = useCallback(async (text: string) => {
    await invoke("copy_text", { text });
    setCopied(true);
    clearTimeout(copyTimerRef.current);
    copyTimerRef.current = setTimeout(() => setCopied(false), 2000);
  }, []);

  const runRedact = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed) {
        setResult(null);
        setError(null);
        return;
      }
      const seq = ++requestSeq.current;
      setLoading(true);
      setError(null);
      try {
        const res = await invoke<RedactionResult>("redact_text", {
          text: trimmed,
        });
        if (seq !== requestSeq.current) return;
        setResult(res);
        if (settings.auto_copy_result) {
          await copy(res.redacted_text);
        }
      } catch (e) {
        if (seq === requestSeq.current) setError(String(e));
      } finally {
        if (seq === requestSeq.current) setLoading(false);
      }
    },
    [copy, settings.auto_copy_result],
  );

  // Ingest text pushed by the shortcut: fill and redact immediately.
  const ingest = useCallback(
    (text: string | null) => {
      clearTimeout(debounceRef.current);
      if (text && text.trim()) {
        setInputText(text);
        void runRedact(text);
      } else {
        setInputText("");
        setResult(null);
        setError(null);
        inputRef.current?.focus();
      }
    },
    [runRedact],
  );

  useEffect(() => {
    // First show: the webview may not have existed when the shortcut fired,
    // so pull the clipboard proactively instead of relying on the event.
    void invoke<string | null>("quick_clipboard").then(ingest);
    const unlisten = listen<string | null>("quick-shown", (e) =>
      ingest(e.payload),
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [ingest]);

  // Esc closes the panel from anywhere.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void invoke("hide_quick_window");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const text = e.target.value;
    setInputText(text);
    clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => void runRedact(text), DEBOUNCE_MS);
  };

  const togglePin = () => {
    const next = !pinned;
    setPinned(next);
    void invoke("set_quick_pinned", { pinned: next });
  };

  return (
    <div className="h-screen flex flex-col select-none cursor-default">
      <div className="flex-1 flex flex-col min-h-0 overflow-y-auto p-3 space-y-2">
        <div className="bg-background border border-mid-gray/20 rounded-lg shrink-0">
          <textarea
            ref={inputRef}
            autoFocus
            className="w-full px-4 py-3 text-sm bg-transparent rounded-lg resize-none h-24 focus:outline-none placeholder:text-mid-gray/70"
            placeholder={t("quick.inputPlaceholder")}
            value={inputText}
            onChange={handleChange}
          />
        </div>

        {error && (
          <div className="px-3 py-2 text-xs text-red-500 bg-red-500/10 border border-red-500/20 rounded-lg shrink-0 animate-fade-in-up">
            {error}
          </div>
        )}

        {loading && !result && (
          <div className="flex-1 flex items-center justify-center text-mid-gray">
            <Spinner />
          </div>
        )}

        {result && (
          <div className="flex-1 flex flex-col min-h-0 space-y-2 animate-fade-in-up">
            <div className="flex items-center justify-between px-4 shrink-0">
              <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
                {t("redactor.outputTitle")}
              </h2>
              <div className="flex items-center gap-3">
                <span className="text-xs text-mid-gray tabular-nums">
                  {result.latency_ms.toFixed(1)}ms
                </span>
                <button
                  className="flex items-center gap-1 text-xs text-mid-gray hover:text-text transition-colors cursor-pointer"
                  onClick={() => void copy(result.redacted_text)}
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
            <div className="flex-1 min-h-0 overflow-y-auto bg-background border border-mid-gray/20 rounded-lg">
              <div className="px-4 py-3 text-sm whitespace-pre-wrap break-words select-text cursor-text leading-relaxed">
                {result.redacted_text}
              </div>
            </div>
            {result.detected_spans.length > 0 && (
              <div className="flex flex-wrap items-center gap-1.5 px-4 shrink-0">
                {result.detected_spans.map((span, i) => (
                  <SpanBadge key={i} category={span.label} />
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="shrink-0 h-9 border-t border-mid-gray/20 flex items-center justify-between px-3">
        <button
          className={`flex items-center gap-1 text-xs transition-colors cursor-pointer ${
            pinned ? "text-background-ui" : "text-mid-gray hover:text-text"
          }`}
          onClick={togglePin}
        >
          {pinned ? (
            <PinOff width={12} height={12} />
          ) : (
            <Pin width={12} height={12} />
          )}
          {t("quick.pin")}
        </button>
      </div>
    </div>
  );
};
