import { useCallback, useEffect, useState } from "react";
import { EntryForm } from "./components/EntryForm";
import { Home } from "./components/Home";
import {
  entriesCreate,
  entriesDelete,
  entriesList,
  entriesUpdate,
  type Entry,
  type MonthView,
} from "./lib/api";
import { currentMonth } from "./lib/dates";
import "./App.css";

type Screen =
  | { name: "home" }
  | { name: "form"; entry: Entry | null };

export default function App() {
  const [month, setMonth] = useState(currentMonth);
  const [view, setView] = useState<MonthView | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [screen, setScreen] = useState<Screen>({ name: "home" });

  const load = useCallback(async (selectedMonth: string) => {
    setLoading(true);
    setError(null);
    try {
      const next = await entriesList(selectedMonth);
      setView(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load(month);
  }, [month, load]);

  if (screen.name === "form") {
    const editing = screen.entry;
    return (
      <main className="app-shell">
        <EntryForm
          entry={editing}
          onCancel={() => setScreen({ name: "home" })}
          onSave={async (draft) => {
            if (editing) {
              await entriesUpdate({ id: editing.id, ...draft });
            } else {
              await entriesCreate(draft);
            }
            setScreen({ name: "home" });
            await load(month);
          }}
          onDelete={
            editing
              ? async () => {
                  await entriesDelete(editing.id);
                  setScreen({ name: "home" });
                  await load(month);
                }
              : undefined
          }
        />
      </main>
    );
  }

  return (
    <main className="app-shell">
      <Home
        month={month}
        view={view}
        loading={loading}
        error={error}
        onMonthChange={setMonth}
        onAdd={() => setScreen({ name: "form", entry: null })}
        onEdit={(entry) => setScreen({ name: "form", entry })}
        onRetry={() => void load(month)}
      />
    </main>
  );
}
