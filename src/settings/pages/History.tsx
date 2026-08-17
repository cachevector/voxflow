import type { PageProps } from "../App";
import { useEffect, useState } from "react";
import { commands } from "@/shared/tauri";
import type { HistoryEntry, VocabularySuggestion } from "@/shared/types";

export default function History({ settings, update }: PageProps) {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [savingCorrection, setSavingCorrection] = useState(false);
  const [suggestion, setSuggestion] = useState<VocabularySuggestion | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

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

  const startCorrection = (entry: HistoryEntry) => {
    setEditingId(entry.id);
    setDraft(entry.text);
    setError(null);
    setNotice(null);
  };

  const saveCorrection = async () => {
    if (!editingId || !draft.trim()) return;
    setSavingCorrection(true);
    setError(null);
    try {
      const result = await commands.correctHistoryEntry(editingId, draft);
      setEntries((current) =>
        current.map((entry) => (entry.id === result.entry.id ? result.entry : entry)),
      );
      setEditingId(null);
      setSuggestion(result.suggestion);
      setNotice("Corrected text copied to the clipboard.");
    } catch (e) {
      setError(String(e));
    } finally {
      setSavingCorrection(false);
    }
  };

  const acceptSuggestion = async () => {
    if (!suggestion) return;
    setError(null);
    try {
      await commands.acceptVocabularySuggestion(suggestion);
      const exists = settings.dictionary.some(
        (entry) =>
          entry.term.toLowerCase() === suggestion.term.toLowerCase() &&
          entry.replacement?.toLowerCase() === suggestion.replacement.toLowerCase(),
      );
      if (!exists) {
        update({
          dictionary: [
            ...settings.dictionary,
            { term: suggestion.term, replacement: suggestion.replacement },
          ],
        });
      }
      setNotice(`VoxFlow will write “${suggestion.replacement}” for “${suggestion.term}”.`);
      setSuggestion(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const dismissSuggestion = async () => {
    if (!suggestion) return;
    setError(null);
    try {
      await commands.dismissVocabularySuggestion(suggestion);
      const exists = settings.vocabulary_suggestion_dismissals.some(
        (dismissal) =>
          dismissal.term.toLowerCase() === suggestion.term.toLowerCase() &&
          dismissal.replacement.toLowerCase() === suggestion.replacement.toLowerCase(),
      );
      if (!exists) {
        update({
          vocabulary_suggestion_dismissals: [
            ...settings.vocabulary_suggestion_dismissals,
            suggestion,
          ],
        });
      }
      setSuggestion(null);
      setNotice("VoxFlow will not suggest that correction again.");
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div>
      <h2 className="mb-4 text-lg font-semibold">History</h2>
      <p className="mb-4 text-sm text-neutral-500">
        Every dictation is saved here. Correct an entry to copy the fixed text and optionally
        teach VoxFlow a spelling.
      </p>
      {loading && <p className="text-sm text-neutral-500">Loading…</p>}
      {error && <p className="text-sm text-red-500">{error}</p>}
      {notice && <p className="mb-3 text-sm text-neutral-500">{notice}</p>}
      {suggestion && (
        <div className="mb-4 rounded-xl border border-accent/30 bg-accent/5 p-4 text-sm">
          <p>
            When VoxFlow hears <strong>“{suggestion.term}”</strong>, write{" "}
            <strong>“{suggestion.replacement}”</strong>?
          </p>
          <div className="mt-3 flex gap-2">
            <button
              type="button"
              className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white"
              onClick={acceptSuggestion}
            >
              Add to vocabulary
            </button>
            <button
              type="button"
              className="rounded-lg bg-neutral-100 px-3 py-1.5 text-xs font-medium dark:bg-neutral-800"
              onClick={dismissSuggestion}
            >
              Not now
            </button>
          </div>
        </div>
      )}
      <ul className="space-y-3">
        {entries.map((e) => (
          <li
            key={e.id}
            className="rounded-xl border border-neutral-200 p-4 dark:border-neutral-800"
          >
            {editingId === e.id ? (
              <div>
                <label className="text-xs text-neutral-500" htmlFor={`history-${e.id}`}>
                  Corrected text
                </label>
                <textarea
                  id={`history-${e.id}`}
                  value={draft}
                  onChange={(event) => setDraft(event.target.value)}
                  className="mt-1 min-h-20 w-full rounded-lg border border-neutral-300 bg-transparent px-3 py-2 text-sm leading-relaxed outline-none focus:border-accent dark:border-neutral-700"
                />
              </div>
            ) : (
              <p className="text-sm leading-relaxed">{e.text}</p>
            )}
            <div className="mt-2 flex flex-wrap gap-2 text-xs text-neutral-500">
              <span>{new Date(e.created_at).toLocaleString()}</span>
              <span>·</span>
              <span>{e.model}</span>
            </div>
            <div className="mt-3 flex gap-2">
              {editingId === e.id ? (
                <>
                  <button
                    type="button"
                    className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white disabled:opacity-50"
                    disabled={savingCorrection || !draft.trim()}
                    onClick={saveCorrection}
                  >
                    {savingCorrection ? "Saving…" : "Save & copy"}
                  </button>
                  <button
                    type="button"
                    className="rounded-lg bg-neutral-100 px-3 py-1.5 text-xs font-medium dark:bg-neutral-800"
                    onClick={() => setEditingId(null)}
                  >
                    Cancel
                  </button>
                </>
              ) : (
                <>
                  <button
                    type="button"
                    className="rounded-lg bg-neutral-100 px-3 py-1.5 text-xs font-medium dark:bg-neutral-800"
                    onClick={() => copy(e.text)}
                  >
                    Copy
                  </button>
                  <button
                    type="button"
                    className="rounded-lg bg-neutral-100 px-3 py-1.5 text-xs font-medium dark:bg-neutral-800"
                    onClick={() => startCorrection(e)}
                  >
                    Correct
                  </button>
                  <button
                    type="button"
                    className="rounded-lg bg-accent/10 px-3 py-1.5 text-xs font-medium text-accent"
                    onClick={() => pasteAgain(e.text)}
                  >
                    Paste again
                  </button>
                </>
              )}
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
