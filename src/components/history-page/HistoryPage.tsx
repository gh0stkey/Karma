import React, { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Copy, Trash2, Check, Eraser } from "lucide-react";
import { SpanBadge } from "../redactor/SpanBadge";
import { useClipboard } from "@/hooks/useClipboard";
import type { HistoryEntry } from "@/lib/types";

const PAGE_SIZE = 30;

export const HistoryPage: React.FC = () => {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [hasMore, setHasMore] = useState(true);
  const sentinelRef = useRef<HTMLDivElement>(null);
  const entriesRef = useRef<HistoryEntry[]>([]);
  const loadingRef = useRef(false);

  useEffect(() => {
    entriesRef.current = entries;
  }, [entries]);

  const loadPage = useCallback(async (cursor?: number) => {
    if (cursor !== undefined && loadingRef.current) return;
    loadingRef.current = true;
    if (cursor === undefined) setLoading(true);
    try {
      const result = await invoke<{
        entries: HistoryEntry[];
        has_more: boolean;
      }>("get_history_entries", { cursor: cursor ?? null, limit: PAGE_SIZE });
      setEntries((prev) =>
        cursor === undefined ? result.entries : [...prev, ...result.entries],
      );
      setHasMore(result.has_more);
    } catch (e) {
      console.error("Failed to load history:", e);
    } finally {
      setLoading(false);
      loadingRef.current = false;
    }
  }, []);

  useEffect(() => {
    loadPage();
  }, [loadPage]);

  useEffect(() => {
    if (loading || !hasMore) return;
    const sentinel = sentinelRef.current;
    if (!sentinel) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          const last = entriesRef.current[entriesRef.current.length - 1];
          if (last) loadPage(last.id);
        }
      },
      { threshold: 0 },
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [loading, hasMore, loadPage]);

  useEffect(() => {
    const unlisten = listen<HistoryEntry>("history-entry-added", (event) => {
      setEntries((prev) => [event.payload, ...prev]);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleDelete = async (id: number) => {
    setEntries((prev) => prev.filter((e) => e.id !== id));
    try {
      await invoke("delete_history_entry", { id });
    } catch (e) {
      console.error("Failed to delete entry:", e);
      loadPage();
    }
  };

  const handleClear = async () => {
    try {
      await invoke("clear_all_history");
      setEntries([]);
    } catch (e) {
      console.error("Failed to clear history:", e);
    }
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <div className="space-y-2">
        <div className="flex items-center justify-between px-4">
          <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
            {t("history.title")}
          </h2>
          {entries.length > 0 && (
            <button
              className="flex items-center gap-1 text-xs text-mid-gray hover:text-text transition-colors cursor-pointer"
              onClick={handleClear}
            >
              <Eraser width={12} height={12} />
              {t("history.clear")}
            </button>
          )}
        </div>
        <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
          {loading ? (
            <div className="px-4 py-8 text-center text-xs text-mid-gray">
              {t("history.loading")}
            </div>
          ) : entries.length === 0 ? (
            <div className="px-4 py-8 text-center text-sm text-mid-gray">
              {t("history.empty")}
            </div>
          ) : (
            <div>
              {entries.map((entry, i) => (
                <React.Fragment key={entry.id}>
                  {i > 0 && <hr className="border-mid-gray/20 mx-4" />}
                  <HistoryEntryRow entry={entry} onDelete={handleDelete} />
                </React.Fragment>
              ))}
              <div ref={sentinelRef} className="h-1" />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

const HistoryEntryRow: React.FC<{
  entry: HistoryEntry;
  onDelete: (id: number) => void;
}> = ({ entry, onDelete }) => {
  const { copied, copy } = useClipboard();

  const handleCopy = async () => {
    await copy(entry.redacted_text);
  };

  return (
    <div className="px-4 py-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs text-mid-gray">{entry.timestamp}</span>
        <div className="flex items-center gap-1">
          <button
            onClick={handleCopy}
            className="p-1 rounded text-text/50 hover:text-logo-primary transition-colors cursor-pointer"
          >
            {copied ? (
              <Check width={14} height={14} />
            ) : (
              <Copy width={14} height={14} />
            )}
          </button>
          <button
            onClick={() => onDelete(entry.id)}
            className="p-1 rounded text-text/50 hover:text-red-500 transition-colors cursor-pointer"
          >
            <Trash2 width={14} height={14} />
          </button>
        </div>
      </div>
      <p className="text-xs text-text/50 truncate select-text cursor-text">
        {entry.input_text}
      </p>
      <p className="text-sm select-text cursor-text">{entry.redacted_text}</p>
      <div className="flex items-center gap-2 flex-wrap">
        {Object.entries(entry.summary).map(([cat, count]) => (
          <SpanBadge key={cat} category={cat} count={count} />
        ))}
        <span className="text-xs text-mid-gray tabular-nums ml-auto">
          {entry.latency_ms.toFixed(1)}ms
        </span>
      </div>
    </div>
  );
};
