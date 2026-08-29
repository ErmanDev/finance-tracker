import { useEffect, useMemo, useState } from "react";
import type { Entry, EntryType } from "../lib/api";
import { todayIsoDate } from "../lib/dates";
import { centavosToPesosInput, pesosToCentavos } from "../lib/money";
import { DeleteDialog } from "./DeleteDialog";

export const CATEGORIES = [
  "Food",
  "Transport",
  "Bills",
  "Salary",
  "Freelance",
  "Other",
] as const;

type Props = {
  entry: Entry | null;
  onCancel: () => void;
  onSave: (draft: {
    type: EntryType;
    amount_centavos: number;
    category: string;
    note: string;
    date: string;
  }) => Promise<void>;
  onDelete?: () => Promise<void>;
};

export function EntryForm({ entry, onCancel, onSave, onDelete }: Props) {
  const editing = entry !== null;
  const [type, setType] = useState<EntryType>(entry?.type ?? "expense");
  const [amount, setAmount] = useState(
    entry ? centavosToPesosInput(entry.amount_centavos) : "",
  );
  const [category, setCategory] = useState(entry?.category ?? "");
  const [note, setNote] = useState(entry?.note ?? "");
  const [date, setDate] = useState(entry?.date ?? todayIsoDate());
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const parsedAmount = useMemo(() => pesosToCentavos(amount), [amount]);

  async function save() {
    if (busy) return;
    const centavos = pesosToCentavos(amount);
    if (centavos === null || centavos <= 0) {
      setError("Enter an amount in pesos, like 250.50.");
      return;
    }
    if (!category.trim()) {
      setError("Pick or type a category.");
      return;
    }
    if (!date) {
      setError("Choose a date.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await onSave({
        type,
        amount_centavos: centavos,
        category: category.trim(),
        note: note.trim(),
        date,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
    }
  }

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (confirmDelete) return;
      if (event.key === "Escape") {
        event.preventDefault();
        if (!busy) onCancel();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [busy, confirmDelete, onCancel]);

  return (
    <section className="form-screen">
      <header className="form-header">
        <p className="eyebrow">{editing ? "Edit entry" : "New entry"}</p>
        <h1>{type === "income" ? "Money in" : "Money out"}</h1>
      </header>

      <form
        className="entry-form"
        onSubmit={(event) => {
          event.preventDefault();
          void save();
        }}
      >
        <fieldset className="type-toggle" aria-label="Entry type">
          <button
            type="button"
            className={type === "income" ? "active income" : ""}
            onClick={() => setType("income")}
          >
            Income
          </button>
          <button
            type="button"
            className={type === "expense" ? "active expense" : ""}
            onClick={() => setType("expense")}
          >
            Expense
          </button>
        </fieldset>

        <label>
          Amount
          <span className="amount-field">
            <span aria-hidden="true">₱</span>
            <input
              autoFocus
              inputMode="decimal"
              placeholder="0.00"
              value={amount}
              onChange={(event) => setAmount(event.target.value)}
              aria-invalid={amount.length > 0 && parsedAmount === null}
            />
          </span>
        </label>

        <label>
          Category
          <input
            list="ledger-categories"
            value={category}
            onChange={(event) => setCategory(event.target.value)}
            placeholder="Food, Transport, Salary…"
          />
          <datalist id="ledger-categories">
            {CATEGORIES.map((item) => (
              <option key={item} value={item} />
            ))}
          </datalist>
        </label>

        <label>
          Note
          <input
            value={note}
            onChange={(event) => setNote(event.target.value)}
            placeholder="Optional"
          />
        </label>

        <label>
          Date
          <input
            type="date"
            value={date}
            onChange={(event) => setDate(event.target.value)}
          />
        </label>

        {error ? <p className="form-error">{error}</p> : null}

        <div className="form-actions">
          <button type="button" className="btn btn-ghost" onClick={onCancel} disabled={busy}>
            Cancel
          </button>
          {editing && onDelete ? (
            <button
              type="button"
              className="btn btn-danger-ghost"
              onClick={() => setConfirmDelete(true)}
              disabled={busy}
            >
              Delete
            </button>
          ) : null}
          <button type="submit" className="btn btn-primary" disabled={busy}>
            {busy ? "Saving…" : "Save"}
          </button>
        </div>
        <p className="kbd-hint">Enter saves · Esc cancels</p>
      </form>

      <DeleteDialog
        open={confirmDelete}
        title="Delete this entry?"
        body="This removes it from the ledger. The monthly totals and all-time balance will update immediately."
        onCancel={() => setConfirmDelete(false)}
        onConfirm={async () => {
          if (!onDelete) return;
          setBusy(true);
          try {
            await onDelete();
          } catch (err) {
            setBusy(false);
            setConfirmDelete(false);
            setError(err instanceof Error ? err.message : String(err));
          }
        }}
        busy={busy}
      />
    </section>
  );
}
