//! Serde mirrors of the JSON returned by `meta.quiltmc.org`.
//!
//! These are wire-format types; see `quilt.rs` for the `extract_*`
//! functions that translate them into the launcher's pivot types.

use serde::Deserialize;

#[derive(Debug, Deserialize,Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuiltMetaData {
    pub id : String,
    pub inherits_from : String,
    #[serde(rename = "type")]
    pub types : String,
    pub main_class : String,
    pub arguments : Game,
    pub libraries : Vec<QuiltLibrary>,
    pub release_time : String,
    pub time : String,

}

#[derive(Debug, Deserialize,Clone)]
pub struct QuiltLibrary {
    pub name : String,
    pub url : String,

}

#[derive(Debug, Deserialize,Clone)]
pub struct Game {
    pub game : Vec<String>
}