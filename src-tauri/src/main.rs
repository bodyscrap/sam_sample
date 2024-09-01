// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use image::{self, GenericImageView, Rgba};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn open_img(path: &str) -> (usize, usize, Vec<u8>) 
{
    let img = image::open(path).expect("failed to open image");
    let (width, height) = img.dimensions();
    let img_src = img.into_rgba8();
    (width as usize, height as usize, img_src.to_vec())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_img])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
