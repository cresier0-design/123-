#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
mod linux_startup;

fn main() {
    // Set WebKit defaults before GTK, WebKit or the async runtime starts threads.
    #[cfg(target_os = "linux")]
    linux_startup::configure();

    crocodile_gena_studio_lib::run();
}
