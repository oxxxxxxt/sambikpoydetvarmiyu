#![cfg_attr(test, allow(dead_code, unused))]

#[cfg(not(test))]
mod commands;
mod domain;
mod error;
mod import;
mod storage;

#[cfg(not(test))]
use std::collections::HashMap;
#[cfg(not(test))]
use std::sync::Mutex;

#[cfg(not(test))]
use storage::db::Database;
#[cfg(not(test))]
use tauri::Manager;

#[cfg(not(test))]
use crate::domain::TrainingSession;

#[cfg(not(test))]
pub struct AppState {
    pub db: Database,
    pub sessions: Mutex<HashMap<String, TrainingSession>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[cfg(not(test))]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db = Database::open(app_data_dir.join("exam-trainer.sqlite3"))
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            app.manage(AppState {
                db,
                sessions: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_bootstrap_state,
            commands::import_seed_docx,
            commands::list_questions,
            commands::start_training,
            commands::submit_answer,
            commands::skip_question,
            commands::finish_training,
            commands::list_mistakes,
            commands::get_statistics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
