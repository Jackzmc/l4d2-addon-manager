// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod commands;

fn main() {
    l4d2_workshop_manager_v2_lib::run()
}
