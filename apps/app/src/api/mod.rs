use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use tauri::{Builder, Runtime};

// REMOVED: pub mod ads;
// REMOVED: pub mod analytics;

pub mod auth;
pub mod cache;
pub mod friends;
pub mod import;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod minecraft_skins;
pub mod mod_plugin;
pub mod mr_auth;
pub mod oauth_utils;
pub mod pack;
pub mod process;
pub mod profile;
pub mod profile_create;
pub mod settings;
pub mod tags;
pub mod utils;
pub mod worlds;

pub type Result<T> = std::result::Result<T, crate::error::Error>;

pub struct Void;

impl Serialize for Void {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let _s = serializer.serialize_struct("Void", 0)?;
        Ok(_s.end()?)
    }
}

pub fn init_all<R: Runtime>(mut builder: Builder<R>) -> Builder<R> {
    builder = builder
        // .plugin(ads::init())       <-- REMOVED
        // .plugin(analytics::init()) <-- REMOVED
        .plugin(auth::init())
        .plugin(cache::init())
        .plugin(friends::init())
        .plugin(import::init())
        .plugin(jre::init())
        .plugin(logs::init())
        .plugin(metadata::init())
        .plugin(minecraft_skins::init())
        .plugin(mod_plugin::init())
        .plugin(mr_auth::init())
        .plugin(pack::init())
        .plugin(process::init())
        .plugin(profile::init())
        .plugin(profile_create::init())
        .plugin(settings::init())
        .plugin(tags::init())
        .plugin(utils::init())
        .plugin(worlds::init());

    builder
}
