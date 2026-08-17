import { useState } from "react";
import type { PageProps } from "../App";
import { Section } from "../components/Field";
import { commands } from "@/shared/tauri";

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

  const restoreSuggestion = async (term: string, replacement: string) => {
    await commands.restoreVocabularySuggestion({ term, replacement });
    update({
      vocabulary_suggestion_dismissals: settings.vocabulary_suggestion_dismissals.filter(
        (dismissal) =>
          !(
            dismissal.term.toLowerCase() === term.toLowerCase() &&
            dismissal.replacement.toLowerCase() === replacement.toLowerCase()
          ),
      ),
    });
  };

  const vocabularyGroups = settings.dictionary.reduce<
    Array<{ preferred: string; entries: Array<{ index: number; term: string }> }>
  >((groups, entry, index) => {
    const preferred = entry.replacement ?? entry.term;
    const group = groups.find(
      (candidate) => candidate.preferred.toLowerCase() === preferred.toLowerCase(),
    );
    if (group) {
      group.entries.push({ index, term: entry.term });
    } else {
      groups.push({ preferred, entries: [{ index, term: entry.term }] });
    }
    return groups;
  }, []);

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
          {vocabularyGroups.map((group) => (
            <li
              key={group.preferred}
              className="rounded-lg border border-neutral-200 px-3 py-2 text-sm dark:border-neutral-800"
            >
              <p className="font-medium">{group.preferred}</p>
              <div className="mt-1 flex flex-wrap gap-2">
                {group.entries.map((entry) => (
                  <span
                    key={`${entry.term}-${entry.index}`}
                    className="inline-flex items-center gap-1 rounded-md bg-neutral-100 px-2 py-1 text-xs dark:bg-neutral-800"
                  >
                    {entry.term}
                    <button
                      type="button"
                      onClick={() => remove(entry.index)}
                      className="text-neutral-500 hover:text-red-500"
                      aria-label={`Remove ${entry.term}`}
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            </li>
          ))}
        </ul>
      )}

      {settings.vocabulary_suggestion_dismissals.length > 0 && (
        <div className="mt-6">
          <p className="text-sm font-medium">Dismissed suggestions</p>
          <p className="mt-1 text-xs text-neutral-500">
            Re-enable a suggestion if you want VoxFlow to offer it after a future correction.
          </p>
          <ul className="mt-2 space-y-2">
            {settings.vocabulary_suggestion_dismissals.map((dismissal) => (
              <li
                key={`${dismissal.term}-${dismissal.replacement}`}
                className="flex items-center justify-between gap-3 rounded-lg border border-neutral-200 px-3 py-2 text-xs dark:border-neutral-800"
              >
                <span>
                  {dismissal.term} → {dismissal.replacement}
                </span>
                <button
                  type="button"
                  onClick={() => restoreSuggestion(dismissal.term, dismissal.replacement)}
                  className="text-accent"
                >
                  Re-enable
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Section>
  );
}
