import React from "react";

export type PIICategory =
  | "private_person"
  | "private_email"
  | "private_phone"
  | "private_address"
  | "private_date"
  | "private_url"
  | "account_number"
  | "secret";

const CATEGORY_LABELS: Record<string, string> = {
  private_person: "PERSON",
  private_email: "EMAIL",
  private_phone: "PHONE",
  private_address: "ADDRESS",
  private_date: "DATE",
  private_url: "URL",
  account_number: "ACCOUNT",
  secret: "SECRET",
};

interface SpanBadgeProps {
  category: string;
  count?: number;
  className?: string;
}

export const SpanBadge: React.FC<SpanBadgeProps> = ({
  category,
  count,
  className = "",
}) => {
  const label = CATEGORY_LABELS[category] ?? category.toUpperCase();

  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-mid-gray/15 text-mid-gray ${className}`}
    >
      {label}
      {count !== undefined && count > 0 && (
        <span className="opacity-70">
          {"\u00D7"}
          {count}
        </span>
      )}
    </span>
  );
};
