// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;
pub mod util;

mod models;

pub mod modules;
mod scan;

fn main() {
    if cfg!(unix) {
        unsafe { std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1"); }
    }
    l4d2_addon_manager_lib::run()
}
