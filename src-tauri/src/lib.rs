mod commands;
mod db;
mod feed;
mod reader;
mod background;

use db::AppDatabase;
use tauri::Manager;

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");

            let database = AppDatabase::new(&app_data_dir)
                .expect("Failed to initialize database");
            app.manage(database);

            // Start background feed refresh scheduler
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                background::scheduler::run(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::feeds::add_feed,
            commands::feeds::get_feeds,
            commands::feeds::refresh_feed,
            commands::feeds::refresh_all_feeds,
            commands::feeds::delete_feed,
            commands::feeds::update_feed,
            commands::feeds::import_opml,
            commands::feeds::export_opml,
            commands::folders::create_folder,
            commands::folders::delete_folder,
            commands::folders::rename_folder,
            commands::articles::get_articles,
            commands::articles::get_article,
            commands::articles::mark_read,
            commands::articles::mark_unread,
            commands::articles::mark_all_read,
            commands::articles::mark_above_read,
            commands::articles::toggle_star,
            commands::articles::toggle_read_later,
            commands::articles::search_articles,
            commands::parser::parse_article,
            commands::tags::create_tag,
            commands::tags::get_tags,
            commands::tags::add_tag_to_article,
            commands::tags::remove_tag_from_article,
            commands::settings::get_setting,
            commands::settings::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Kublai Reader");
}
