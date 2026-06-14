//! BenchBoss library root.
//!
//! Wires together:
//!   - `controller` — Raw Input enumeration, capture, ViGEm output
//!   - `routing`    — the routing table (slot ↔ device handle mapping)
//!   - `commands`   — Tauri IPC commands
//!   - `AppState`   — shared state managed by Tauri

pub mod commands;
pub mod controller;
pub mod error;
pub mod routing;

use controller::{
    capture::start_capture,
    enumerate::enumerate_gamepads,
    vigem::{create_targets, ViGEmTargets},
    GamepadState, PhysicalDevice,
};
use routing::RoutingTable;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

/// Application-wide shared state, managed by Tauri.
pub struct AppState {
    /// List of detected physical devices (updated by the capture thread).
    pub devices: Arc<RwLock<Vec<PhysicalDevice>>>,
    /// Latest gamepad state per device handle (updated by capture thread).
    pub states: Arc<RwLock<HashMap<u64, GamepadState>>>,
    /// Current routing table — which physical handle maps to which virtual slot.
    pub routing: Arc<RwLock<RoutingTable>>,
    /// The 4 virtual Xbox controllers.
    pub vigem: Arc<Mutex<ViGEmTargets>>,
    /// Set to `true` while the forwarding loop is running.
    pub forwarding: Arc<Mutex<bool>>,
}

/// Build the Tauri app, initialise subsystems, and run the event loop.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // ---- Initialise ViGEm (4 virtual controllers) ----
    let vigem = match create_targets() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to initialise ViGEmBus: {e}");
            // On non-Windows or if ViGEm is not installed, use the stub.
            // create_targets() always returns Ok on non-Windows.
            panic!("Cannot continue without ViGEm: {e}");
        }
    };

    // ---- Enumerate initial device list ----
    let initial_devices = enumerate_gamepads().unwrap_or_else(|e| {
        log::warn!("Initial enumeration failed: {e}");
        vec![]
    });
    log::info!("Found {} gamepad(s) at startup", initial_devices.len());

    // ---- Shared state ----
    let devices: Arc<RwLock<Vec<PhysicalDevice>>> = Arc::new(RwLock::new(initial_devices));
    let states: Arc<RwLock<HashMap<u64, GamepadState>>> = Arc::new(RwLock::new(HashMap::new()));
    let routing: Arc<RwLock<RoutingTable>> = Arc::new(RwLock::new(RoutingTable::default()));

    // ---- Start Raw Input capture (+ XInput polling on Windows) ----
    start_capture(Arc::clone(&states), Arc::clone(&devices));

    let app_state = AppState {
        devices,
        states,
        routing,
        vigem: Arc::new(Mutex::new(vigem)),
        forwarding: Arc::new(Mutex::new(false)),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::assign_player,
            commands::assign_slot,
            commands::assign_bench,
            commands::swap,
            commands::get_routing_table,
            commands::start_forwarding,
            commands::stop_forwarding,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
