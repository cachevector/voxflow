import { useState } from "react";
import type { PageProps } from "../App";
import { Section } from "../components/Field";

export default function Dictionary({ settings, update }: PageProps) {
  const [term, setTerm] = useState("");
  const [replacement, setReplacement] = useState("");

  const add = () => {
    const t = term.trim();
    if (!t) return;
    const r = replacement.trim();
    update({
      dictionary: [
        ...settings.dictionary,
        { term: t, replacement: r.length > 0 ? r : null },
      ],
    });
    setTerm("");
    setReplacement("");
  };

  const remove = (index: number) => {
    update({ dictionary: settings.dictionary.filter((_, i) => i !== index) });
  };

  return (
    <Section title="Vocabulary">
      <p className="text-sm text-neutral-500">
        Preferred spellings are fed to Whisper so it is more likely to write{" "}
        <em>handoff</em> instead of <em>hand of</em>. A replacement is applied
        after transcription for a specific mishear (e.g.{" "}
        <code className="text-xs">lead code</code> →{" "}
        <code className="text-xs">LeetCode</code>).
      </p>

      <div className="flex flex-wrap items-end gap-2">
        <label className="min-w-[10rem] flex-1 text-xs text-neutral-500">
          Term
          <input
            value={term}
            onChange={(e) => setTerm(e.target.value)}
            placeholder="handoff"
            className="mt-1 w-full rounded-lg border border-neutral-300 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-accent dark:border-neutral-700"
          />
        </label>
        <label className="min-w-[10rem] flex-1 text-xs text-neutral-500">
          Replace mishear (optional)
          <input
            value={replacement}
            onChange={(e) => setReplacement(e.target.value)}
            placeholder="lead code → LeetCode"
            className="mt-1 w-full rounded-lg border border-neutral-300 bg-transparent px-3 py-1.5 text-sm outline-none focus:border-accent dark:border-neutral-700"
          />
        </label>
        <button
          type="button"
          onClick={add}
          className="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white"
        >
          Add
        </button>
      </div>

      {settings.dictionary.length === 0 ? (
        <p className="text-xs text-neutral-500">
          Built-in software terms (LeetCode, Zellij, Ghostty, Remotion, …) are
          already biased. Add personal names and project-specific words here.
        </p>
      ) : (
        <ul className="space-y-2">
          {settings.dictionary.map((entry, i) => (
            <li
              key={`${entry.term}-${i}`}
              className="flex items-center justify-between gap-3 rounded-lg border border-neutral-200 px-3 py-2 text-sm dark:border-neutral-800"
            >
              <span>
                <span className="font-medium">{entry.term}</span>
                {entry.replacement && (
                  <span className="text-neutral-500">
                    {" "}
                    → {entry.replacement}
                  </span>
                )}
              </span>
              <button
                type="button"
                onClick={() => remove(i)}
                className="text-xs text-neutral-500 hover:text-red-500"
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}
    </Section>
  );
}
