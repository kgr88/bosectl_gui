use crate::{commands, state::AppState};

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_backend_info,
            commands::discover_devices,
            commands::connect_device,
            commands::disconnect_device,
            commands::get_device_snapshot,
            commands::set_noise_control,
            commands::set_equalizer,
            commands::set_device_setting,
            commands::get_profiles,
            commands::set_profile,
            commands::delete_profile,
            commands::enter_pairing,
            commands::power_off,
        ])
        .run(tauri::generate_context!())
        .expect("error while running bosectl");
}
