#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backend_client;
mod commands;
mod contracts;
mod lifecycle;
mod state;

fn main() {
    lifecycle::run();
}
