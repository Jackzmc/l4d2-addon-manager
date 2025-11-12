// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod util;

mod models;

mod scan;
pub mod modules;

fn main() {
    l4d2_addon_manager_lib::run()
}
