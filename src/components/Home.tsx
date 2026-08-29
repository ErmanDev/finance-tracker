import { formatEntryDate, formatMonthLabel, shiftMonth } from "../lib/dates";
import { formatPhp } from "../lib/money";
import type { Entry, MonthView } from "../lib/api";

type Props = {
  month: string;
  view: MonthView | null;
  loading: boolean;
  error: string | null;
  onMonthChange: (month: string) => void;
  onAdd: () => void;
  onEdit: (entry: Entry) => void;
  onRetry: () => void;
};

function Stat({
  label,
  value,
  tone,
}: {
  label: string;
  value: number | null;
  tone?: "income" | "expense" | "muted";
}) {
  return (
    <div className="stat">
      <span className="stat-label">{label}</span>
      <span className={`stat-value ${tone ?? ""}`}>
        {value === null ? "—" : formatPhp(value)}
      </span>
    </div>
  );
}

export function Home({
  month,
  view,
  loading,
  error,
  onMonthChange,
  onAdd,
  onEdit,
  onRetry,
}: Props) {
  const entries = view?.entries ?? [];
  const empty = !loading && !error && entries.length === 0;

  return (
    <section className="home">
      <div className="home-brand">
        <img
          className="home-logo"
          src="/logo.png"
          alt="Ledger"
          width={128}
          height={128}
        />
      </div>
      <header className="home-header">
        <div>
          <p className="eyebrow">Ledger</p>
          <h1>Philippine Peso books</h1>
        </div>
        <div className="month-nav">
          <button
            type="button"
            className="icon-btn"
            aria-label="Previous month"
            onClick={() => onMonthChange(shiftMonth(month, -1))}
          >
            ‹
          </button>
          <label className="month-label">
            <span className="sr-only">Month</span>
            <input
              type="month"
              value={month}
              onChange={(event) => {
                if (event.target.value) onMonthChange(event.target.value);
              }}
            />
            <strong>{formatMonthLabel(month)}</strong>
          </label>
          <button
            type="button"
            className="icon-btn"
            aria-label="Next month"
            onClick={() => onMonthChange(shiftMonth(month, 1))}
          >
            ›
          </button>
        </div>
      </header>

      <div className="stats">
        <Stat
          label="Month income"
          value={view ? view.month_income_centavos : null}
          tone="income"
        />
        <Stat
          label="Month expense"
          value={view ? view.month_expense_centavos : null}
          tone="expense"
        />
        <Stat
          label="Month net"
          value={view ? view.month_net_centavos : null}
          tone={(view?.month_net_centavos ?? 0) < 0 ? "expense" : "income"}
        />
        <Stat
          label="All-time balance"
          value={view ? view.all_time_balance_centavos : null}
        />
      </div>

      <div className="list-panel">
        <div className="list-heading">
          <h2>Entries</h2>
          <button type="button" className="btn btn-primary add-btn" onClick={onAdd}>
            +
            <span>Add</span>
          </button>
        </div>

        {loading ? <p className="status">Loading this month…</p> : null}
        {error ? (
          <div className="status error">
            <p>{error}</p>
            <button type="button" className="btn btn-ghost" onClick={onRetry}>
              Try again
            </button>
          </div>
        ) : null}
        {empty ? (
          <div className="status empty">
            <img
              className="home-logo home-logo-empty"
              src="/logo.png"
              alt=""
              width={96}
              height={96}
            />
            <p>No entries this month.</p>
          </div>
        ) : null}

        {entries.length > 0 ? (
          <ul className="entry-list">
            {entries.map((entry) => (
              <li key={entry.id}>
                <button type="button" className="entry-row" onClick={() => onEdit(entry)}>
                  <span className="entry-date">{formatEntryDate(entry.date)}</span>
                  <span className="entry-main">
                    <span className="entry-category">{entry.category}</span>
                    {entry.note ? <span className="entry-note">{entry.note}</span> : null}
                  </span>
                  <span className={`entry-amount ${entry.type}`}>
                    {entry.type === "expense" ? "−" : ""}
                    {formatPhp(entry.amount_centavos).replace(/^−/, "")}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  );
}
