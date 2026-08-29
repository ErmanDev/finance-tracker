import { invoke } from "@tauri-apps/api/core";

export type EntryType = "income" | "expense";

export type Entry = {
  id: number;
  type: EntryType;
  amount_centavos: number;
  category: string;
  note: string;
  date: string;
  created_at: string;
};

export type NewEntry = {
  type: EntryType;
  amount_centavos: number;
  category: string;
  note: string;
  date: string;
};

export type UpdateEntry = NewEntry & { id: number };

export type MonthView = {
  entries: Entry[];
  month_income_centavos: number;
  month_expense_centavos: number;
  month_net_centavos: number;
  all_time_balance_centavos: number;
};

export function entriesList(month: string): Promise<MonthView> {
  return invoke("entries_list", { month });
}

export function entriesCreate(entry: NewEntry): Promise<Entry> {
  return invoke("entries_create", { entry });
}

export function entriesUpdate(entry: UpdateEntry): Promise<Entry> {
  return invoke("entries_update", { entry });
}

export function entriesDelete(id: number): Promise<void> {
  return invoke("entries_delete", { id });
}
