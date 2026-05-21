export interface SectionTab<T extends string> {
  id: T;
  label: string;
}

interface SectionTabsProps<T extends string> {
  tabs: SectionTab<T>[];
  activeTab: T;
  onTabChange: (tab: T) => void;
}

export function SectionTabs<T extends string>({
  tabs,
  activeTab,
  onTabChange,
}: SectionTabsProps<T>) {
  return (
    <div
      className="flex items-center gap-6"
      role="tablist"
      aria-orientation="horizontal"
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeTab;

        return (
          <button
            key={tab.id}
            type="button"
            role="tab"
            aria-selected={isActive}
            className={`relative pb-2 text-[15px] leading-none font-medium transition-colors duration-150 ${
              isActive
                ? "text-text"
                : "text-mid-gray hover:text-text/80"
            }`}
            onClick={() => onTabChange(tab.id)}
          >
            <span>{tab.label}</span>
            <span
              className={`absolute inset-x-0 -bottom-px h-0.5 rounded-full bg-logo-primary transition-opacity duration-150 ${
                isActive ? "opacity-100" : "opacity-0"
              }`}
            />
          </button>
        );
      })}
    </div>
  );
}