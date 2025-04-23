mod ashita;
mod check_game;
mod commands;
mod config;
mod file_download;
mod state;
mod task_manager;
mod tasks;
mod util;
mod windower;

use state::AppStateData;
use tauri::{async_runtime::RwLock, Manager};
use tauri_plugin_window_state::StateFlags;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("xi_launcher_lib=debug")
        .init();

    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        commands::get_profiles,
        commands::save_profile,
        commands::delete_profile,
        commands::duplicate_profile,
        commands::move_profile,
        commands::should_request_password,
        commands::check_launch_profile,
        commands::install_game_for_profile,
        commands::update_profile_server_files,
        commands::launch_profile,
        commands::cancel_possible_profile_task,
        commands::list_ashita_addons,
        commands::list_ashita_plugins,
    ]);

    #[cfg(debug_assertions)]
    {
        use specta_typescript::{BigIntExportBehavior, Typescript};
        // Only export on non-release builds
        let mut config = Typescript::default();
        config = config.bigint(BigIntExportBehavior::Number);

        specta_builder
            .export(config, "../src/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    // Allow unused mut, since mutable is necessary during release
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::POSITION)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .invoke_handler(tauri::generate_handler![
            commands::get_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::duplicate_profile,
            commands::move_profile,
            commands::should_request_password,
            commands::check_launch_profile,
            commands::install_game_for_profile,
            commands::update_profile_server_files,
            commands::launch_profile,
            commands::cancel_possible_profile_task,
            commands::list_ashita_addons,
            commands::list_ashita_plugins,
        ])
        .setup(move |app| {
            specta_builder.mount_events(app);

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            let app_state_data = tauri::async_runtime::block_on(AppStateData::new(app))?;
            let app_state = RwLock::new(app_state_data);
            app.manage(app_state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
