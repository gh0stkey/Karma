import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { SectionTabs } from "@/components/ui/SectionTabs";
import { RedactorEditorTab } from "./RedactorEditorTab";
import { RedactorHistoryTab } from "./RedactorHistoryTab";

type RedactorTab = "redact" | "history";

export const RedactorPage: React.FC = () => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<RedactorTab>("redact");

  const tabs = [
    { id: "redact", label: t("redactor.tabs.redact") },
    { id: "history", label: t("redactor.tabs.history") },
  ] satisfies { id: RedactorTab; label: string }[];

  return (
    <div className="max-w-3xl w-full mx-auto flex flex-col gap-3">
      <div className="flex justify-start px-4 pb-2 border-b border-mid-gray/15">
        <SectionTabs
          tabs={tabs}
          activeTab={activeTab}
          onTabChange={setActiveTab}
        />
      </div>

      {activeTab === "redact" ? <RedactorEditorTab /> : <RedactorHistoryTab />}
    </div>
  );
};
