use crate::api::Health;

#[tauri::command]
pub fn health() -> Health {
    Health { ok: true }
}
