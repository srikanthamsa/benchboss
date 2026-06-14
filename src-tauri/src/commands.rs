//! Tauri IPC commands exposed to the frontend.

use crate::{
    AppState,
    controller::PhysicalDevice,
    error::AppResult,
    routing::RoutingTable,
};
use tauri::State;

/// Return the current list of detected physical controllers.
#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> AppResult<Vec<PhysicalDevice>> {
    let devices = state.devices.read().unwrap();
    Ok(devices.clone())
}

/// Assign a human-readable player name to a physical device.
#[tauri::command]
pub async fn assign_player(
    state: State<'_, AppState>,
    handle: u64,
    name: String,
) -> AppResult<()> {
    let mut devices = state.devices.write().unwrap();
    if let Some(dev) = devices.iter_mut().find(|d| d.handle == handle) {
        dev.player_name = Some(name);
        Ok(())
    } else {
        Err(format!("No device with handle {handle:#x}").into())
    }
}

/// Route a physical device into an active game slot (0-3).
#[tauri::command]
pub async fn assign_slot(
    state: State<'_, AppState>,
    slot: usize,
    handle: u64,
) -> AppResult<()> {
    if slot >= 4 {
        return Err(format!("Slot {slot} is out of range (0-3)").into());
    }
    let mut routing = state.routing.write().unwrap();
    routing.assign_slot(slot, handle);
    Ok(())
}

/// Route a physical device to the bench.
#[tauri::command]
pub async fn assign_bench(state: State<'_, AppState>, handle: u64) -> AppResult<()> {
    let mut routing = state.routing.write().unwrap();
    routing.assign_bench(handle);
    Ok(())
}

/// Swap two device handles anywhere in the routing table (slot ↔ slot or slot ↔ bench).
/// This is the hot-swap operation — the game never sees a disconnect.
#[tauri::command]
pub async fn swap(
    state: State<'_, AppState>,
    handle_a: u64,
    handle_b: u64,
) -> AppResult<()> {
    let mut routing = state.routing.write().unwrap();
    routing.swap(handle_a, handle_b);
    log::info!("Swapped {handle_a:#x} ↔ {handle_b:#x}");
    Ok(())
}

/// Return the current routing table snapshot.
#[tauri::command]
pub async fn get_routing_table(state: State<'_, AppState>) -> AppResult<RoutingTable> {
    let routing = state.routing.read().unwrap();
    Ok(routing.clone())
}

/// Start the input→ViGEm forwarding loop.
#[tauri::command]
pub async fn start_forwarding(state: State<'_, AppState>) -> AppResult<()> {
    let already_running = {
        let guard = state.forwarding.lock().unwrap();
        *guard
    };

    if already_running {
        log::info!("start_forwarding: already running");
        return Ok(());
    }

    *state.forwarding.lock().unwrap() = true;

    let states = std::sync::Arc::clone(&state.states);
    let routing = std::sync::Arc::clone(&state.routing);
    let vigem = std::sync::Arc::clone(&state.vigem);
    let forwarding_flag = std::sync::Arc::clone(&state.forwarding);

    tokio::spawn(async move {
        log::info!("Forwarding loop started");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(4));

        loop {
            interval.tick().await;

            // Check stop flag
            if !*forwarding_flag.lock().unwrap() {
                log::info!("Forwarding loop stopped");
                break;
            }

            // Snapshot routing table and gamepad states
            let table_snapshot = {
                let r = routing.read().unwrap();
                r.clone()
            };
            let states_snapshot = {
                let s = states.read().unwrap();
                s.clone()
            };

            // Push each slot's state to the corresponding virtual controller
            let mut vigem_guard = vigem.lock().unwrap();
            for (slot_index, maybe_handle) in table_snapshot.slots.iter().enumerate() {
                let state_to_push = maybe_handle
                    .and_then(|h| states_snapshot.get(&h).cloned())
                    .unwrap_or_default();

                if let Err(e) = vigem_guard.update_slot(slot_index, &state_to_push) {
                    log::warn!("update_slot({slot_index}) failed: {e}");
                }
            }
        }
    });

    Ok(())
}

/// Stop the input→ViGEm forwarding loop.
#[tauri::command]
pub async fn stop_forwarding(state: State<'_, AppState>) -> AppResult<()> {
    *state.forwarding.lock().unwrap() = false;
    log::info!("Forwarding stop requested");
    Ok(())
}
