import type { CSSProperties } from "react";
import type { StatusChipTone } from "@/lib/statusChips";

type StatusChipProps = {
  icon?: string;
  label: string;
  tone?: StatusChipTone;
  style?: CSSProperties;
  title?: string;
};

function chipToneClassName(tone: StatusChipTone): string {
  switch (tone) {
    case "accent":
      return "amux-chip amux-chip--accent";
    case "approval":
      return "amux-chip amux-chip--approval";
    case "warning":
      return "amux-chip amux-chip--warning";
    case "success":
      return "amux-chip amux-chip--success";
    case "danger":
      return "amux-chip amux-chip--danger";
    default:
      return "amux-chip";
  }
}

export function StatusChip({
  icon,
  label,
  tone = "neutral",
  style,
  title,
}: StatusChipProps) {
  return (
    <span
      className={chipToneClassName(tone)}
      style={{ fontSize: 11, padding: "2px 8px", ...style }}
      title={title}
    >
      {icon ? <span aria-hidden="true">{icon}</span> : null}
      <span>{label}</span>
    </span>
  );
}
