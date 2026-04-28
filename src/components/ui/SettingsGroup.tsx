import React from "react";

interface SettingsGroupProps {
  title?: string;
  description?: string;
  children: React.ReactNode;
}

export const SettingsGroup: React.FC<SettingsGroupProps> = ({
  title,
  description,
  children,
}) => {
  const items = React.Children.toArray(children).filter(Boolean);

  return (
    <div className="space-y-2">
      {title && (
        <div className="px-4">
          <h2 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
            {title}
          </h2>
          {description && (
            <p className="text-xs text-mid-gray mt-1">{description}</p>
          )}
        </div>
      )}
      <div className="bg-background border border-mid-gray/20 rounded-lg overflow-visible">
        {items.map((child, i) => (
          <React.Fragment key={i}>
            {i > 0 && <hr className="border-mid-gray/20 mx-4" />}
            {child}
          </React.Fragment>
        ))}
      </div>
    </div>
  );
};
