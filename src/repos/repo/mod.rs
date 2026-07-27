use serde::Deserialize;

pub mod mws;
pub mod z77;

#[derive(Clone, Deserialize)]
pub struct SearchEntry {
    pub id: String,
    pub name: String,
    pub desc: String,
}
