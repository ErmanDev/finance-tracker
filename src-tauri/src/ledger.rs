use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub id: i64,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub amount_centavos: i64,
    pub category: String,
    pub note: String,
    pub date: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub amount_centavos: i64,
    pub category: String,
    #[serde(default)]
    pub note: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntry {
    pub id: i64,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub amount_centavos: i64,
    pub category: String,
    #[serde(default)]
    pub note: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MonthView {
    pub entries: Vec<Entry>,
    pub month_income_centavos: i64,
    pub month_expense_centavos: i64,
    pub month_net_centavos: i64,
    pub all_time_balance_centavos: i64,
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY,
            type TEXT NOT NULL CHECK (type IN ('income', 'expense')),
            amount_centavos INTEGER NOT NULL,
            category TEXT NOT NULL,
            note TEXT DEFAULT '',
            date TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

pub fn now_local_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn validate_type(entry_type: &str) -> Result<(), String> {
    match entry_type {
        "income" | "expense" => Ok(()),
        _ => Err("type must be income or expense".into()),
    }
}

fn validate_date(date: &str) -> Result<(), String> {
    if date.len() != 10 {
        return Err("date must be YYYY-MM-DD".into());
    }
    let bytes = date.as_bytes();
    let ok = bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                true
            } else {
                c.is_ascii_digit()
            }
        });
    if !ok {
        return Err("date must be YYYY-MM-DD".into());
    }
    let month: u8 = date[5..7]
        .parse()
        .map_err(|_| "date must be YYYY-MM-DD".to_string())?;
    let day: u8 = date[8..10]
        .parse()
        .map_err(|_| "date must be YYYY-MM-DD".to_string())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("date must be a valid calendar day".into());
    }
    Ok(())
}

fn validate_month(month: &str) -> Result<(), String> {
    if month.len() != 7 || month.as_bytes()[4] != b'-' {
        return Err("month must be YYYY-MM".into());
    }
    if !month.as_bytes().iter().enumerate().all(|(i, c)| {
        if i == 4 {
            true
        } else {
            c.is_ascii_digit()
        }
    }) {
        return Err("month must be YYYY-MM".into());
    }
    let m: u8 = month[5..7]
        .parse()
        .map_err(|_| "month must be YYYY-MM".to_string())?;
    if !(1..=12).contains(&m) {
        return Err("month must be YYYY-MM".into());
    }
    Ok(())
}

fn validate_amount(amount_centavos: i64) -> Result<(), String> {
    if amount_centavos <= 0 {
        Err("amount must be greater than zero centavos".into())
    } else {
        Ok(())
    }
}

fn validate_category(category: &str) -> Result<(), String> {
    if category.trim().is_empty() {
        Err("category is required".into())
    } else {
        Ok(())
    }
}

fn get_entry(conn: &Connection, id: i64) -> Result<Entry, String> {
    conn.query_row(
        "SELECT id, type, amount_centavos, category, note, date, created_at
         FROM entries WHERE id = ?1",
        params![id],
        row_to_entry,
    )
    .optional()
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("entry {id} not found"))
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        entry_type: row.get(1)?,
        amount_centavos: row.get(2)?,
        category: row.get(3)?,
        note: row.get(4)?,
        date: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn sum_type_in_month(conn: &Connection, entry_type: &str, month: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COALESCE(SUM(amount_centavos), 0) FROM entries
         WHERE type = ?1 AND substr(date, 1, 7) = ?2",
        params![entry_type, month],
        |row| row.get::<_, i64>(0),
    )
    .map_err(|e| e.to_string())
}

fn all_time_balance(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT COALESCE(SUM(CASE WHEN type = 'income' THEN amount_centavos ELSE 0 END), 0)
              - COALESCE(SUM(CASE WHEN type = 'expense' THEN amount_centavos ELSE 0 END), 0)
         FROM entries",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map_err(|e| e.to_string())
}

pub fn create_entry(conn: &Connection, new_entry: &NewEntry) -> Result<Entry, String> {
    validate_type(&new_entry.entry_type)?;
    validate_amount(new_entry.amount_centavos)?;
    validate_category(&new_entry.category)?;
    validate_date(&new_entry.date)?;

    let created_at = now_local_iso();
    conn.execute(
        "INSERT INTO entries (type, amount_centavos, category, note, date, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            new_entry.entry_type,
            new_entry.amount_centavos,
            new_entry.category.trim(),
            new_entry.note,
            new_entry.date,
            created_at
        ],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    get_entry(conn, id)
}

pub fn update_entry(conn: &Connection, update: &UpdateEntry) -> Result<Entry, String> {
    validate_type(&update.entry_type)?;
    validate_amount(update.amount_centavos)?;
    validate_category(&update.category)?;
    validate_date(&update.date)?;

    let changed = conn
        .execute(
            "UPDATE entries
             SET type = ?1, amount_centavos = ?2, category = ?3, note = ?4, date = ?5
             WHERE id = ?6",
            params![
                update.entry_type,
                update.amount_centavos,
                update.category.trim(),
                update.note,
                update.date,
                update.id
            ],
        )
        .map_err(|e| e.to_string())?;

    if changed == 0 {
        return Err(format!("entry {} not found", update.id));
    }
    get_entry(conn, update.id)
}

pub fn delete_entry(conn: &Connection, id: i64) -> Result<(), String> {
    let changed = conn
        .execute("DELETE FROM entries WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if changed == 0 {
        return Err(format!("entry {id} not found"));
    }
    Ok(())
}

pub fn list_month(conn: &Connection, month: &str) -> Result<MonthView, String> {
    validate_month(month)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, type, amount_centavos, category, note, date, created_at
             FROM entries
             WHERE substr(date, 1, 7) = ?1
             ORDER BY date DESC, id DESC",
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![month], row_to_entry)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let month_income_centavos = sum_type_in_month(conn, "income", month)?;
    let month_expense_centavos = sum_type_in_month(conn, "expense", month)?;

    Ok(MonthView {
        entries,
        month_income_centavos,
        month_expense_centavos,
        month_net_centavos: month_income_centavos - month_expense_centavos,
        all_time_balance_centavos: all_time_balance(conn)?,
    })
}

pub fn count_entries(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
        .map_err(|e| e.to_string())
}

/// Sample August 2026 books. Only used when LEDGER_SEED=1 and the table is empty.
pub fn seed_sample_month(conn: &Connection) -> Result<(), String> {
    if count_entries(conn)? > 0 {
        return Ok(());
    }

    let rows: [(&str, i64, &str, &str, &str); 8] = [
        ("income", 4_500_000, "Salary", "August payroll", "2026-08-01"),
        ("expense", 245_050, "Food", "Weekend market", "2026-08-03"),
        ("expense", 120_000, "Transport", "Jeep and train", "2026-08-05"),
        ("expense", 680_000, "Bills", "Electricity and water", "2026-08-08"),
        ("expense", 89_025, "Food", "Lunch near the office", "2026-08-12"),
        ("income", 850_000, "Freelance", "Illustration invoice", "2026-08-15"),
        ("expense", 50_000, "Other", "Barber", "2026-08-20"),
        ("expense", 35_000, "Transport", "Airport taxi", "2026-08-22"),
    ];

    for (entry_type, amount_centavos, category, note, date) in rows {
        create_entry(
            conn,
            &NewEntry {
                entry_type: entry_type.into(),
                amount_centavos,
                category: category.into(),
                note: note.into(),
                date: date.into(),
            },
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn income(centavos: i64, date: &str) -> NewEntry {
        NewEntry {
            entry_type: "income".into(),
            amount_centavos: centavos,
            category: "Salary".into(),
            note: String::new(),
            date: date.into(),
        }
    }

    fn expense(centavos: i64, date: &str, category: &str) -> NewEntry {
        NewEntry {
            entry_type: "expense".into(),
            amount_centavos: centavos,
            category: category.into(),
            note: String::new(),
            date: date.into(),
        }
    }

    #[test]
    fn create_edit_delete_and_month_sums_use_centavos() {
        let conn = mem();

        let payday = create_entry(&conn, &income(10_050, "2026-08-10")).unwrap();
        assert_eq!(payday.amount_centavos, 10_050);
        assert_eq!(payday.entry_type, "income");

        let lunch = create_entry(&conn, &expense(255, "2026-08-11", "Food")).unwrap();
        assert_eq!(lunch.amount_centavos, 255);

        // Another month must not change August totals, but does change all-time balance.
        create_entry(&conn, &income(50_000, "2026-07-01")).unwrap();

        let august = list_month(&conn, "2026-08").unwrap();
        assert_eq!(august.entries.len(), 2);
        assert_eq!(august.entries[0].id, lunch.id, "newest date first");
        assert_eq!(august.entries[1].id, payday.id);
        assert_eq!(august.month_income_centavos, 10_050);
        assert_eq!(august.month_expense_centavos, 255);
        assert_eq!(august.month_net_centavos, 10_050 - 255);
        assert_eq!(august.all_time_balance_centavos, 10_050 - 255 + 50_000);

        let july = list_month(&conn, "2026-07").unwrap();
        assert_eq!(july.entries.len(), 1);
        assert_eq!(july.month_income_centavos, 50_000);
        assert_eq!(july.month_expense_centavos, 0);
        assert_eq!(july.month_net_centavos, 50_000);
        assert_eq!(july.all_time_balance_centavos, august.all_time_balance_centavos);

        let edited = update_entry(
            &conn,
            &UpdateEntry {
                id: lunch.id,
                entry_type: "expense".into(),
                amount_centavos: 1_500,
                category: "Transport".into(),
                note: "Grab".into(),
                date: "2026-08-11".into(),
            },
        )
        .unwrap();
        assert_eq!(edited.amount_centavos, 1_500);
        assert_eq!(edited.category, "Transport");

        let after_edit = list_month(&conn, "2026-08").unwrap();
        assert_eq!(after_edit.month_expense_centavos, 1_500);
        assert_eq!(after_edit.month_net_centavos, 10_050 - 1_500);
        assert_eq!(
            after_edit.all_time_balance_centavos,
            10_050 - 1_500 + 50_000
        );

        delete_entry(&conn, payday.id).unwrap();
        let after_delete = list_month(&conn, "2026-08").unwrap();
        assert_eq!(after_delete.entries.len(), 1);
        assert_eq!(after_delete.month_income_centavos, 0);
        assert_eq!(after_delete.month_expense_centavos, 1_500);
        assert_eq!(after_delete.month_net_centavos, -1_500);
        assert_eq!(after_delete.all_time_balance_centavos, 50_000 - 1_500);
    }

    #[test]
    fn empty_month_is_zero_and_newest_id_breaks_date_ties() {
        let conn = mem();
        let empty = list_month(&conn, "2026-08").unwrap();
        assert!(empty.entries.is_empty());
        assert_eq!(empty.month_income_centavos, 0);
        assert_eq!(empty.month_expense_centavos, 0);
        assert_eq!(empty.month_net_centavos, 0);
        assert_eq!(empty.all_time_balance_centavos, 0);

        let first = create_entry(&conn, &expense(100, "2026-08-28", "Food")).unwrap();
        let second = create_entry(&conn, &expense(200, "2026-08-28", "Transport")).unwrap();
        let view = list_month(&conn, "2026-08").unwrap();
        assert_eq!(view.entries[0].id, second.id);
        assert_eq!(view.entries[1].id, first.id);
        assert_eq!(view.month_expense_centavos, 300);
    }

    #[test]
    fn rejects_float_like_invalid_inputs() {
        let conn = mem();
        let err = create_entry(&conn, &income(0, "2026-08-01")).unwrap_err();
        assert!(err.contains("greater than zero"));
        assert!(create_entry(
            &conn,
            &NewEntry {
                entry_type: "transfer".into(),
                amount_centavos: 100,
                category: "Other".into(),
                note: String::new(),
                date: "2026-08-01".into(),
            }
        )
        .is_err());
        assert!(create_entry(&conn, &income(100, "08-01-2026")).is_err());
        assert!(list_month(&conn, "2026-13").is_err());
        assert!(update_entry(
            &conn,
            &UpdateEntry {
                id: 999,
                entry_type: "income".into(),
                amount_centavos: 100,
                category: "Salary".into(),
                note: String::new(),
                date: "2026-08-01".into(),
            }
        )
        .is_err());
        assert!(delete_entry(&conn, 999).is_err());
    }

    #[test]
    fn seed_inserts_only_when_empty() {
        let conn = mem();
        seed_sample_month(&conn).unwrap();
        seed_sample_month(&conn).unwrap();
        assert_eq!(count_entries(&conn).unwrap(), 8);
        let august = list_month(&conn, "2026-08").unwrap();
        assert_eq!(august.month_income_centavos, 4_500_000 + 850_000);
        assert_eq!(
            august.month_expense_centavos,
            245_050 + 120_000 + 680_000 + 89_025 + 50_000 + 35_000
        );
        assert_eq!(
            august.month_net_centavos,
            august.month_income_centavos - august.month_expense_centavos
        );
        assert_eq!(
            august.all_time_balance_centavos,
            august.month_net_centavos
        );
    }
}
