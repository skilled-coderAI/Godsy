mod commands;

#[allow(clippy::missing_panics_doc)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::list_plans,
            commands::get_plan,
            commands::run_plan,
            commands::list_kb_files,
            commands::upload_kb_file,
            commands::delete_kb_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Godsy");
}
