import type { ReactNode } from "react";

type StatsCardProps = {
  label: string;
  value: ReactNode;
  tone?: "default" | "good" | "bad" | "warn";
};

export function StatsCard({ label, value, tone = "default" }: StatsCardProps) {
  return (
    <div className={`stats-card stats-card--${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
