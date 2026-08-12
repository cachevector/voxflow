import type { ReactNode } from "react";

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mb-8">
      <h2 className="mb-4 text-lg font-semibold">{title}</h2>
      <div className="space-y-4">{children}</div>
    </section>
  );
}

export function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 rounded-lg border border-neutral-200 p-3 dark:border-neutral-800">
      <div>
        <div className="text-sm font-medium">{label}</div>
        {hint && <div className="text-xs text-neutral-500">{hint}</div>}
      </div>
      <div>{children}</div>
    </div>
  );
}

export function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`h-6 w-10 rounded-pill transition-colors ${checked ? "bg-accent" : "bg-neutral-300 dark:bg-neutral-700"}`}
    >
      <span
        className={`block h-5 w-5 translate-y-0.5 rounded-full bg-white transition-transform ${
          checked ? "translate-x-[18px]" : "translate-x-0.5"
        }`}
      />
    </button>
  );
}

export function TextInput(props: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className={`rounded-lg border border-neutral-300 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-accent dark:border-neutral-700 ${props.className ?? ""}`}
    />
  );
}

export function Select({
  value,
  onChange,
  options,
}: {
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="rounded-lg border border-neutral-300 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-accent dark:border-neutral-700"
    >
      {options.map((o) => (
        <option key={o.value} value={o.value}>
          {o.label}
        </option>
      ))}
    </select>
  );
}
