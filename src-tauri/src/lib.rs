mod db;
mod stats;

use chrono::{Local, NaiveDate};
use db::{Category, CategoryInput, Completion, Db, Result};
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::{Manager, State};

type AppDb<'a> = State<'a, Mutex<Db>>;

#[tauri::command]
fn list_categories(db: AppDb) -> Result<Vec<Category>> {
    db.lock().unwrap().categories()
}

#[tauri::command]
fn create_category(db: AppDb, input: CategoryInput) -> Result<Category> {
    db.lock().unwrap().create_category(&input)
}

#[tauri::command]
fn update_category(db: AppDb, id: i64, input: CategoryInput) -> Result<()> {
    db.lock().unwrap().update_category(id, &input)
}

#[tauri::command]
fn delete_category(db: AppDb, id: i64) -> Result<()> {
    db.lock().unwrap().delete_category(id)
}

#[tauri::command]
fn get_completions(db: AppDb, from: NaiveDate, to: NaiveDate) -> Result<Vec<Completion>> {
    db.lock().unwrap().completions_between(from, to)
}

#[tauri::command]
fn toggle_completion(db: AppDb, category_id: i64, date: NaiveDate) -> Result<bool> {
    db.lock().unwrap().toggle_completion(category_id, date)
}

#[tauri::command]
fn get_stats(db: AppDb) -> Result<stats::Stats> {
    let db = db.lock().unwrap();
    let categories = db.categories()?;
    let done: Vec<HashSet<NaiveDate>> = categories
        .iter()
        .map(|c| db.completion_dates(c.id))
        .collect::<Result<_>>()?;
    let tracked: Vec<stats::Tracked> = categories
        .iter()
        .zip(&done)
        .map(|(category, done)| stats::Tracked { category, done })
        .collect();
    Ok(stats::compute(&tracked, Local::now().date_naive()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db = Db::open(&dir.join("focus.db"))?;
            app.manage(Mutex::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_categories,
            create_category,
            update_category,
            delete_category,
            get_completions,
            toggle_completion,
            get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar Focus");
}
