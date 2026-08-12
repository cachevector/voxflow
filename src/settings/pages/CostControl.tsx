import { useEffect, useState } from "react";
import type { PageProps } from "../App";
import { Row, Section, TextInput } from "../components/Field";
import { commands } from "@/shared/tauri";
import type { CostDashboard } from "@/shared/types";

export default function CostControl({ settings, update }: PageProps) {
  const [dashboard, setDashboard] = useState<CostDashboard | null>(null);

  useEffect(() => {
    commands.getCostDashboard().then(setDashboard).catch(() => setDashboard(null));
  }, []);

  return (
    <Section title="Cost Control">
      {dashboard && (
        <div className="grid grid-cols-2 gap-3">
          <Stat label="Minutes transcribed" value={dashboard.minutes_transcribed.toFixed(1)} />
          <Stat label="Estimated spend" value={`$${dashboard.estimated_usd.toFixed(2)}`} />
          <Stat label="Projected monthly" value={`$${dashboard.projected_monthly_usd.toFixed(2)}`} />
          <Stat label="Self-hosted %" value={`${dashboard.self_hosted_percentage.toFixed(0)}%`} />
        </div>
      )}
      {dashboard?.cap_warnings.map((w) => (
        <p key={w} className="text-xs text-amber-600 dark:text-amber-400">
          {w}
        </p>
      ))}

      <Row label="Monthly spend cap (USD)">
        <TextInput
          type="number"
          value={settings.cost_control.monthly_spend_cap_usd ?? ""}
          onChange={(e) =>
            update({
              cost_control: {
                ...settings.cost_control,
                monthly_spend_cap_usd: e.target.value ? Number(e.target.value) : null,
              },
            })
          }
          className="w-28"
        />
      </Row>
      <Row label="Monthly minute cap">
        <TextInput
          type="number"
          value={settings.cost_control.monthly_minute_cap ?? ""}
          onChange={(e) =>
            update({
              cost_control: {
                ...settings.cost_control,
                monthly_minute_cap: e.target.value ? Number(e.target.value) : null,
              },
            })
          }
          className="w-28"
        />
      </Row>
    </Section>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-neutral-200 p-3 dark:border-neutral-800">
      <div className="text-xs text-neutral-500">{label}</div>
      <div className="text-lg font-semibold">{value}</div>
    </div>
  );
}
