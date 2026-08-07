// Hide the console window on Windows release builds: a tray app that opens a
// black terminal behind itself looks broken.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    portify_app_lib::run()
}
