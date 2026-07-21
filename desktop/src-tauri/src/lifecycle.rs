use std::{ffi::c_void, sync::atomic::Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

use crate::{commands, state::AppState};

pub struct SingleInstanceGuard {
    #[cfg(windows)]
    handle: *mut c_void,
}

#[cfg(windows)]
unsafe impl Send for SingleInstanceGuard {}
#[cfg(windows)]
unsafe impl Sync for SingleInstanceGuard {}

#[cfg(windows)]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { CloseHandle(self.handle) };
        }
    }
}

#[cfg(all(test, windows))]
impl SingleInstanceGuard {
    pub(crate) fn for_test() -> Self { Self { handle: std::ptr::null_mut() } }
}

#[cfg(all(test, not(windows)))]
impl SingleInstanceGuard {
    pub(crate) fn for_test() -> Self { Self {} }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attributes: *mut c_void, initial_owner: i32, name: *const u16) -> *mut c_void;
    fn GetLastError() -> u32;
    fn CloseHandle(handle: *mut c_void) -> i32;
}

#[cfg(windows)]
fn acquire_single_instance() -> Result<SingleInstanceGuard, ()> {
    const ERROR_ALREADY_EXISTS: u32 = 183;
    let name: Vec<u16> = "Local\\bosectl-gui-single-instance\0".encode_utf16().collect();
    let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr()) };
    if handle.is_null() || unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        if !handle.is_null() {
            unsafe { CloseHandle(handle) };
        }
        Err(())
    } else {
        Ok(SingleInstanceGuard { handle })
    }
}

#[cfg(not(windows))]
fn acquire_single_instance() -> Result<SingleInstanceGuard, ()> {
    Ok(SingleInstanceGuard {})
}

fn open_controls(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let handle = app.clone();
    std::thread::spawn(move || {
        let _ = WebviewWindowBuilder::new(&handle, "main", WebviewUrl::App("index.html".into()))
            .title("bosectl")
            .inner_size(780.0, 620.0)
            .min_inner_size(520.0, 420.0)
            .center()
            .build();
    });
}

fn tray_image() -> tauri::image::Image<'static> {
    const SIZE: usize = 32;
    let mut pixels = vec![0_u8; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as i32 - 15;
            let dy = y as i32 - 15;
            if dx * dx + dy * dy <= 14 * 14 {
                let offset = (y * SIZE + x) * 4;
                pixels[offset..offset + 4].copy_from_slice(&[24, 89, 201, 255]);
                let vertical = (10..=13).contains(&x) && (7..=24).contains(&y);
                let bowls = ((12..=21).contains(&x) && (7..=15).contains(&y))
                    || ((12..=22).contains(&x) && (16..=24).contains(&y));
                let hollow = ((14..=18).contains(&x) && (10..=12).contains(&y))
                    || ((14..=19).contains(&x) && (19..=21).contains(&y));
                if vertical || (bowls && !hollow && (x >= 19 || y == 7 || y == 15 || y == 16 || y == 24)) {
                    pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
    }
    tauri::image::Image::new_owned(pixels, SIZE as u32, SIZE as u32)
}

pub fn run() {
    let single_instance = match acquire_single_instance() {
        Ok(guard) => guard,
        Err(()) => return,
    };
    let state = AppState::new(single_instance);
    let app = tauri::Builder::default()
        .manage(state)
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
        ])
        .setup(|app| {
            app.state::<AppState>().backend.attach_app(app.handle().clone());
            let open = MenuItem::with_id(app, "open", "Open controls", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            TrayIconBuilder::new()
                .icon(tray_image())
                .tooltip("bosectl")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => open_controls(app),
                    "quit" => {
                        app.state::<AppState>().quitting.store(true, Ordering::Release);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if matches!(event, TrayIconEvent::DoubleClick { button: MouseButton::Left, .. }) {
                        open_controls(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building bosectl");

    app.run(|app, event| match event {
        RunEvent::WindowEvent { label, event: WindowEvent::CloseRequested { api, .. }, .. }
            if label == "main" && !app.state::<AppState>().quitting.load(Ordering::Acquire) =>
        {
            api.prevent_close();
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.destroy();
            }
        }
        RunEvent::ExitRequested { api, .. } => {
            let state = app.state::<AppState>();
            if !state.shutdown_complete.load(Ordering::Acquire) {
                api.prevent_exit();
                let backend = state.backend.clone();
                let handle = app.clone();
                std::thread::spawn(move || {
                    backend.shutdown();
                    handle.state::<AppState>().shutdown_complete.store(true, Ordering::Release);
                    handle.exit(0);
                });
            }
        }
        _ => {}
    });
}
