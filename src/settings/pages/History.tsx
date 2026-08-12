import type { PageProps } from "../App";
import { useEffect, useState } from "react";
import { commands } from "@/shared/tauri";
import type { HistoryEntry } from "@/shared/types";

export default function History(_props: PageProps) {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    commands
      .listHistory(100)
      .then(setEntries)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  const copy = async (text: string) => {
    await navigator.clipboard.writeText(text);
  };

  const pasteAgain = async (text: string) => {
    await commands.pasteText(text);
  };

  return (
    <div>
      <h2 className="mb-4 text-lg font-semibold">History</h2>
      <p className="mb-4 text-sm text-neutral-500">
        Every dictation is saved here. Copy or paste again if insertion missed the cursor.
      </p>
      {loading && <p className="text-sm text-neutral-500">Loading…</p>}
      {error && <p className="text-sm text-red-500">{error}</p>}
      <ul className="space-y-3">
        {entries.map((e) => (
          <li
            key={e.id}
            className="rounded-xl border border-neutral-200 p-4 dark:border-neutral-800"
          >
            <p className="text-sm leading-relaxed">{e.text}</p>
            <div className="mt-2 flex flex-wrap gap-2 text-xs text-neutral-500">
              <span>{new Date(e.created_at).toLocaleString()}</span>
              <span>·</span>
              <span>{e.model}</span>
            </div>
            <div className="mt-3 flex gap-2">
              <button
                type="button"
                className="rounded-lg bg-neutral-100 px-3 py-1.5 text-xs font-medium dark:bg-neutral-800"
                onClick={() => copy(e.text)}
              >
                Copy
              </button>
              <button
                type="button"
                className="rounded-lg bg-accent/10 px-3 py-1.5 text-xs font-medium text-accent"
                onClick={() => pasteAgain(e.text)}
              >
                Paste again
              </button>
            </div>
          </li>
        ))}
      </ul>
      {!loading && entries.length === 0 && (
        <p className="text-sm text-neutral-500">No dictations yet.</p>
      )}
    </div>
  );
}
