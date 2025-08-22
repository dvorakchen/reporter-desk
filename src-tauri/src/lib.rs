pub mod config;
pub mod director;
pub mod mpeg;
pub mod news;
pub mod subtitle;
pub mod tts;
pub mod video;
pub mod image;

use crate::config::GlobalConfig;
use crate::director::Director;
use crate::director::source::SourceName;
use crate::news::NewsTitle;
use log::{LevelFilter, info};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::RwLock;

#[tauri::command]
async fn get_hot_news_list(
    source: SourceName,
    app: AppHandle,
    config: State<'_, RwLock<GlobalConfig>>,
) -> Result<Vec<NewsTitle>, ()> {
    info!("get_hot_news_list -- source: {}", source);
    let config_g = config.read().await;
    let ali_api_key = config_g.get_ali_dashscope_api_key();
    let deepseek_api_key = config_g.get_deepseek_api_key();
    info!("get_hot_news_list -- ali api key: {}", ali_api_key);
    info!(
        "get_hot_news_list -- deepseek api key: {}",
        deepseek_api_key
    );

    let director = Director::default(ali_api_key, deepseek_api_key, app);

    let list = director.get_hot_news_list(&source).await;
    info!("get_hot_news_list -- list length: {}", list.len());

    Ok(list)
}

#[tauri::command]
async fn gen_video(
    news_title: NewsTitle,
    app: AppHandle,
    config: State<'_, RwLock<GlobalConfig>>,
) -> Result<String, String> {
    info!("get_video --- news_title: {:?}", news_title);

    let config_g = config.read().await;

    let deepseek_key = config_g.get_deepseek_api_key();
    let ali_key = config_g.get_ali_dashscope_api_key();
    info!("get_video -- deepseek api key: {}", deepseek_key);
    info!("get_video -- ali key api key: {}", ali_key);

    let director = Director::default(ali_key, deepseek_key, app);

    let res = director
        .shot_single(news_title)
        .await
        .map_err(|e| e.to_string())?;

    Ok(res.path.canonicalize().unwrap().display().to_string())
}

#[tauri::command]
async fn get_config(config: State<'_, RwLock<GlobalConfig>>) -> Result<[String; 2], ()> {
    let config_g = config.read().await;
    let ds_key = config_g.get_deepseek_api_key();
    let ali_key = config_g.get_ali_dashscope_api_key();
    info!("get_config -- deepseek api key: {}", ds_key);
    info!("get_config -- ali key api key: {}", ali_key);

    Ok([ds_key, ali_key])
}

#[tauri::command]
async fn set_config(
    ds_key: String,
    ali_key: String,
    config: State<'_, RwLock<GlobalConfig>>,
) -> Result<(), ()> {
    info!(
        "set_config -- deepseek key: {}, ali key: {}",
        ds_key, ali_key
    );
    let mut config_g = config.write().await;
    config_g.set_ali_dashscope_api_key(ali_key);
    config_g.set_deepseek_api_key(ds_key);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_plugin = {
        let mut log_builder = tauri_plugin_log::Builder::new().targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
        ]);
        if cfg!(debug_assertions) {
            log_builder = log_builder.level(LevelFilter::Trace);
        } else {
            log_builder = log_builder.level(LevelFilter::Info);
        }

        log_builder.build()
    };

    tauri::Builder::default()
        .plugin(log_plugin)
        .setup(|app| {
            app.manage(RwLock::new(crate::config::GlobalConfig::new()));
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_hot_news_list,
            gen_video,
            get_config,
            set_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
