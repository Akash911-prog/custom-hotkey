// #![windows_subsystem = "windows"]

use std::thread;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use win_hotkeys::{HotkeyManager, VKey};

pub mod force_quit;

fn main() {
    let mut hotkey_manager: HotkeyManager<()> = HotkeyManager::new();

    hotkey_manager
        .register_hotkey(VKey::Q, &[VKey::LWin], move || {
            force_quit::force_quit();
        })
        .unwrap();

    thread::spawn(move || {
        hotkey_manager.event_loop();
    });

    let event_loop = EventLoopBuilder::new().build();

    // 1. Create the menu and give the item a unique ID
    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append(&quit_i);

    let icon = tray_icon::Icon::from_path("F:\\projects\\custom-hotkey\\icon.ico", Some((32, 32)))
        .unwrap();

    // 2. Create the tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(tray_menu))
        .with_tooltip("My Rust Tray App")
        .build()
        .unwrap();

    // 3. Get the global receiver for menu events
    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // 4. Handle menu events
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
