import React from "react";
import { useTranslation } from "react-i18next";
import { Button } from "./Button";

interface PathDisplayProps {
  path: string;
  onOpen: () => void;
  disabled?: boolean;
  openLabel?: string;
}

export const PathDisplay: React.FC<PathDisplayProps> = ({
  path,
  onOpen,
  disabled = false,
  openLabel,
}) => {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2">
      <div className="flex-1 min-w-0 px-2 py-1 bg-mid-gray/10 border border-mid-gray/80 rounded-md text-sm font-mono break-all select-text cursor-text">
        {path}
      </div>
      <Button
        onClick={onOpen}
        variant="secondary"
        size="sm"
        disabled={disabled}
      >
        {openLabel || t("settings.storage.open")}
      </Button>
    </div>
  );
};
