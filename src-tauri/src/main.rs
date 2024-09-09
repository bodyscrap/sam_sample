// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use image::{self, GenericImageView, Rgba};
mod segment_anything;
use segment_anything::SamApp;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn open_img(path: &str) -> (usize, usize, Vec<u8>) 
{
    let img = image::open(path).expect("failed to open image");
    let (width, height) = img.dimensions();
    let img_src = img.into_rgba8();
    (width as usize, height as usize, img_src.to_vec())
}

#[tauri::command]
fn process_sam(width:u32, height:u32, data: Vec<u8>, px:f32, py:f32, sam_app:tauri::State<'_, segment_anything::SamApp>)
-> (u32, u32, Vec<u8>) 
{
    // 入力画像からマスクを生成
    let mask = sam_app.process_sam(width, height, data, px, py).unwrap();
    (width, height, mask.into_vec())
}

fn main() {
    tauri::Builder::default()
        // Stateの初期化
        .setup(|app| {
            let sam_app = SamApp::new_tyny("model").expect("failed to initialize sam app");
            app.manage(sam_app);
            Ok(())
        })
        // ハンドラの登録
        .invoke_handler(tauri::generate_handler![open_img, process_sam])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
