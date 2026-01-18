#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![recursion_limit = "256"]

use native_dialog::{DialogBuilder, MessageLevel};
use std::env;
use tauri::{Listener, Manager};
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(feature = "updater")]
mod updater_impl;
#[cfg(not(feature = "updater"))]
mod updater_impl_noop;

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    tracing::info!("Initializing app event state...");
    theseus::EventState::init(app.clone()).await?;

    tracing::info!("Initializing app state...");
    State::init().await?;

    let state = State::get().await?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir(), true)?;
    app.asset_protocol_scope()
        .allow_directory(state.directories.caches_dir().join("icons"), true)?;

    Ok(())
}

// Should be called once Vue has mounted the app
#[tracing::instrument(skip_all)]
#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    let win = app.get_window("main").unwrap();
    if let Err(e) = win.show() {
        DialogBuilder::new()
            .title("Error while showing window")
            .message(&format!("{e:#?}"))
            .level(MessageLevel::Error)
            .show()
            .expect("error while showing error dialog");
    }
    
    // Force focus
    win.set_focus().unwrap();
}

fn main() {
    // MODIFICATION: Removed Sentry initialization for privacy/lightweight build
    
    // Initialize logging
    tracing_subscriber::fmt::init();

    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(macos::init());
    }

    // MODIFICATION: Removed .plugin(sentry_tauri::plugin())

    builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
        // ... Existing single instance logic ...
        app.emit_to("main", "single-instance", argv).unwrap();
        let win = app.get_window("main").unwrap();
        win.set_focus().unwrap();
        if !win.is_visible().unwrap() {
            win.show().unwrap();
        }
    }));

    builder = builder
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_persisted_scope::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            show_window,
            #[cfg(feature = "updater")]
            updater_impl::check_update,
            #[cfg(feature = "updater")]
            updater_impl::install_update,
            #[cfg(not(feature = "updater"))]
            updater_impl_noop::check_update,
            #[cfg(not(feature = "updater"))]
            updater_impl_noop::install_update,
        ]);

    // Initialize APIs (This calls api/mod.rs)
    builder = api::init_all(builder);

    let result = builder.run(tauri::generate_context!());

    if let Err(e) = result {
        tracing::error!("Error while running tauri application: {:?}", e);

        #[cfg(target_os = "windows")]
        {
            if format!("{e:?}").contains("Runtime(CreateWebview(WebView2Error(WindowsError") {
                DialogBuilder::new()
                    .set_level(MessageLevel::Error)
                    .set_title("Initialization error")
                    .set_text("Your Microsoft Edge WebView2 installation is corrupt.\n\nMicrosoft Edge WebView2 is required to run Modrinth App.\n\nLearn how to repair it at https://support.modrinth.com/en/articles/8797765-corrupted-microsoft-edge-webview2-installation")
                    .show()
                    .unwrap();
                panic!("webview2 initialization failed");
            }
        }

        DialogBuilder::new()
            .set_level(MessageLevel::Error)
            .set_title("Initialization error")
            .set_text(&format!("Cannot initialize application due to an error:\n{e:?}"))
            .show()
            .unwrap();

        panic!("{}: {:?}", "error while running tauri application", e);
    }
}
