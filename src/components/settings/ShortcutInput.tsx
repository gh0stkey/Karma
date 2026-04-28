import React, { useEffect, useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import {
  getKeyName,
  formatKeyCombination,
  normalizeKey,
} from "../../lib/utils/keyboard";
import { ResetButton } from "../ui/ResetButton";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import { useOsType } from "../../hooks/useOsType";
import { toast } from "sonner";

const DEFAULT_SHORTCUT = "command+shift+k";

interface ShortcutInputProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ShortcutInput: React.FC<ShortcutInputProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { settings, updateSetting } = useSettings();
  const [keyPressed, setKeyPressed] = useState<string[]>([]);
  const [recordedKeys, setRecordedKeys] = useState<string[]>([]);
  const [editing, setEditing] = useState(false);
  const [originalBinding, setOriginalBinding] = useState<string>("");
  const shortcutRef = useRef<HTMLDivElement | null>(null);
  const osType = useOsType();

  useEffect(() => {
    if (!editing) return;

    let cleanup = false;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (cleanup) return;
      if (e.repeat) return;
      e.preventDefault();

      const rawKey = getKeyName(e, osType);
      const key = normalizeKey(rawKey);

      if (!keyPressed.includes(key)) {
        setKeyPressed((prev) => [...prev, key]);
        if (!recordedKeys.includes(key)) {
          setRecordedKeys((prev) => [...prev, key]);
        }
      }
    };

    const handleKeyUp = async (e: KeyboardEvent) => {
      if (cleanup) return;
      e.preventDefault();

      const rawKey = getKeyName(e, osType);
      const key = normalizeKey(rawKey);

      setKeyPressed((prev) => prev.filter((k) => k !== key));

      const updatedKeyPressed = keyPressed.filter((k) => k !== key);
      if (updatedKeyPressed.length === 0 && recordedKeys.length > 0) {
        const modifiers = [
          "ctrl",
          "control",
          "shift",
          "alt",
          "option",
          "meta",
          "command",
          "cmd",
          "super",
          "win",
          "windows",
        ];
        const sortedKeys = recordedKeys.sort((a, b) => {
          const aIsModifier = modifiers.includes(a.toLowerCase());
          const bIsModifier = modifiers.includes(b.toLowerCase());
          if (aIsModifier && !bIsModifier) return -1;
          if (!aIsModifier && bIsModifier) return 1;
          return 0;
        });
        const newShortcut = sortedKeys.join("+");

        try {
          await invoke("update_global_shortcut", {
            oldShortcut: originalBinding,
            newShortcut,
          });
          updateSetting("global_shortcut", newShortcut);
        } catch (error) {
          console.error("Failed to change shortcut:", error);
          toast.error(
            t("settings.general.shortcut.errors.set", {
              error: String(error),
            }),
          );

          if (originalBinding) {
            try {
              await invoke("update_global_shortcut", {
                oldShortcut: newShortcut,
                newShortcut: originalBinding,
              });
            } catch (resetError) {
              console.error("Failed to reset shortcut:", resetError);
            }
          }
        }

        setEditing(false);
        setKeyPressed([]);
        setRecordedKeys([]);
        setOriginalBinding("");
      }
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (cleanup) return;
      if (shortcutRef.current && !shortcutRef.current.contains(e.target as Node)) {
        setEditing(false);
        setKeyPressed([]);
        setRecordedKeys([]);
        setOriginalBinding("");
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("click", handleClickOutside);

    return () => {
      cleanup = true;
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("click", handleClickOutside);
    };
  }, [
    keyPressed,
    recordedKeys,
    editing,
    originalBinding,
    updateSetting,
    osType,
    t,
  ]);

  const startRecording = () => {
    if (editing) return;
    setOriginalBinding(settings.global_shortcut || "");
    setEditing(true);
    setKeyPressed([]);
    setRecordedKeys([]);
  };

  const formatCurrentKeys = (): string => {
    if (recordedKeys.length === 0)
      return t("settings.general.shortcut.pressKeys");
    return formatKeyCombination(recordedKeys.join("+"), osType);
  };

  const handleReset = async () => {
    const oldShortcut = settings.global_shortcut || "";
    try {
      await invoke("update_global_shortcut", {
        oldShortcut,
        newShortcut: DEFAULT_SHORTCUT,
      });
      updateSetting("global_shortcut", DEFAULT_SHORTCUT);
    } catch (error) {
      console.error("Failed to reset shortcut:", error);
      toast.error(String(error));
    }
  };

  return (
    <SettingContainer
      title={t("settings.general.shortcut.title")}
      description={t("settings.general.shortcut.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex items-center space-x-1">
        {editing ? (
          <div
            ref={shortcutRef}
            className="px-2 py-1 text-sm font-semibold border border-logo-primary bg-logo-primary/30 rounded-md"
          >
            {formatCurrentKeys()}
          </div>
        ) : (
          <div
            className="px-2 py-1 text-sm font-semibold bg-mid-gray/10 border border-mid-gray/80 hover:bg-logo-primary/10 rounded-md cursor-pointer hover:border-logo-primary"
            onClick={startRecording}
          >
            {formatKeyCombination(settings.global_shortcut || "", osType)}
          </div>
        )}
        <ResetButton onClick={handleReset} />
      </div>
    </SettingContainer>
  );
};
