mod ledger;

use std::fs;
use std::sync::Mutex;

use ledger::{list_month, NewEntry, UpdateEntry};
use rusqlite::Connection;
use tauri::{Manager, State};

pub struct AppState {
    db: Mutex<Connection>,
}

fn open_db(app: &tauri::App) -> Result<Connection, Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("ledger.sqlite");
    let conn = Connection::open(&path)?;
    ledger::init_schema(&conn)?;
    if std::env::var("LEDGER_SEED").ok().as_deref() == Some("1") {
        ledger::seed_sample_month(&conn).map_err(|e| e.to_string())?;
    }
    Ok(conn)
}

#[tauri::command]
fn entries_list(state: State<AppState>, month: String) -> Result<ledger::MonthView, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    list_month(&conn, &month)
}

#[tauri::command]
fn entries_create(state: State<AppState>, entry: NewEntry) -> Result<ledger::Entry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ledger::create_entry(&conn, &entry)
}

#[tauri::command]
fn entries_update(state: State<AppState>, entry: UpdateEntry) -> Result<ledger::Entry, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ledger::update_entry(&conn, &entry)
}

#[tauri::command]
fn entries_delete(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    ledger::delete_entry(&conn, id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = open_db(app).expect("failed to open ledger database");
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            entries_list,
            entries_create,
            entries_update,
            entries_delete
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ledger");
}
