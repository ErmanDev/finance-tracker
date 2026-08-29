# Ledger

A local Philippine Peso (₱) cashbook. One window, one SQLite file, no accounts and no cloud.

Amounts are stored as integer centavos. The app never keeps pesos as floating-point numbers.

## Clone from Origin

```bash
origin repo clone <org-or-user>/ledger
cd ledger
```

If you already have the repository locally, skip clone and continue from the project root.

## Prerequisites

- **Node.js** 20 or newer (npm is fine)
- **Rust** stable, with `cargo` on your `PATH`
- **Tauri 2 Linux packages** on Debian/Ubuntu:

  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf
  ```

  On macOS, Xcode command-line tools are enough. On Windows, use the Tauri MSVC stack.

## Run

```bash
npm install
npm run tauri dev
```

The window opens at 1100×720. The database is created on first launch in the app data directory (`ledger.sqlite`). Closing and reopening the app keeps every entry.

## Tests

Rust unit tests cover create, edit, delete, month sums, and all-time balance using integer centavos:

```bash
npm test
```

## Sample data (debug only)

Seed one sample month only when you set `LEDGER_SEED=1`. It is off by default, and it inserts nothing if the ledger already has rows.

```bash
LEDGER_SEED=1 npm run tauri dev
```

## What v1 does

- **Home** — month income, month expense, month net, all-time balance, newest-first list, month filter
- **Entry form** — income or expense, amount, category, note, date. Same screen for add and edit. Esc cancels, Enter saves. Edit shows Delete.
- **Delete** — confirm dialog, not a separate route

Out of v1: cloud, accounts, sync, multi-currency, CSV import, budgets, recurring entries, attachments, and translations.
