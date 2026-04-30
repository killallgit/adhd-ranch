use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Health {
    pub ok: bool,
}
