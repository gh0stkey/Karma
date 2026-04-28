import React from "react";
import { useTranslation } from "react-i18next";
import { ShieldCheck, Cpu, Globe, History, Cog, Info } from "lucide-react";
import KarmaLogo from "./icons/KarmaLogo";
import { RedactorPage } from "./redactor/RedactorPage";
import { ModelPage } from "./model-page/ModelPage";
import { ServerPage } from "./server-page/ServerPage";
import { HistoryPage } from "./history-page/HistoryPage";
import { SettingsPage } from "./settings/SettingsPage";
import { AboutPage } from "./about-page/AboutPage";

export type SidebarSection = keyof typeof SECTIONS_CONFIG;

interface IconProps {
  width?: number | string;
  height?: number | string;
  size?: number | string;
  className?: string;
  [key: string]: any;
}

interface SectionConfig {
  labelKey: string;
  icon: React.ComponentType<IconProps>;
  component: React.ComponentType;
  enabled: () => boolean;
}

export const SECTIONS_CONFIG = {
  redactor: {
    labelKey: "sidebar.redactor",
    icon: ShieldCheck,
    component: RedactorPage,
    enabled: () => true,
  },
  model: {
    labelKey: "sidebar.model",
    icon: Cpu,
    component: ModelPage,
    enabled: () => true,
  },
  server: {
    labelKey: "sidebar.server",
    icon: Globe,
    component: ServerPage,
    enabled: () => true,
  },
  history: {
    labelKey: "sidebar.history",
    icon: History,
    component: HistoryPage,
    enabled: () => true,
  },
  settings: {
    labelKey: "sidebar.settings",
    icon: Cog,
    component: SettingsPage,
    enabled: () => true,
  },
  about: {
    labelKey: "sidebar.about",
    icon: Info,
    component: AboutPage,
    enabled: () => true,
  },
} as const satisfies Record<string, SectionConfig>;

interface SidebarProps {
  activeSection: SidebarSection;
  onSectionChange: (section: SidebarSection) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeSection,
  onSectionChange,
}) => {
  const { t } = useTranslation();

  const sections = Object.entries(SECTIONS_CONFIG)
    .filter(([_, config]) => config.enabled())
    .map(([id, config]) => ({ id: id as SidebarSection, ...config }));

  return (
    <div className="flex flex-col w-40 h-full border-e border-mid-gray/20 items-center px-2">
      <KarmaLogo width={48} className="m-4" />
      <div className="flex flex-col w-full items-center gap-1 pt-2 border-t border-mid-gray/20">
        {sections.map((section) => {
          const Icon = section.icon;
          const isActive = activeSection === section.id;

          return (
            <div
              key={section.id}
              className={`flex gap-2 items-center p-2 w-full rounded-lg cursor-pointer transition-colors ${
                isActive
                  ? "bg-logo-primary/80"
                  : "hover:bg-mid-gray/20 hover:opacity-100 opacity-85"
              }`}
              onClick={() => onSectionChange(section.id)}
            >
              <Icon width={24} height={24} className="shrink-0" />
              <p
                className="text-sm font-medium truncate"
                title={t(section.labelKey)}
              >
                {t(section.labelKey)}
              </p>
            </div>
          );
        })}
      </div>
    </div>
  );
};
