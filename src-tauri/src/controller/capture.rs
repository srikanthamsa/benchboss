//! Raw Input message-pump thread + per-device gamepad state.
//!
//! # Architecture
//!
//! A background OS thread runs a Win32 hidden-window message pump.  It
//! registers for Raw Input with RIDEV_INPUTSINK so it receives events even
//! when the app is not focused.  On WM_INPUT it records which device sent
//! data and marks it "active".
//!
//! # Known limitation (Phase 2 TODO)
//!
//! Parsing raw HID reports is highly device-specific and would require
//! reading per-device HID descriptor preparsed data for every report field.
//! Parsec virtual controllers may use non-standard report layouts.
//!
//! For the Phase 1 PoC we rely on a **parallel XInput polling thread** that
//! reads axis/button values via the XInput API (indices 0-3 correspond to
//! the first four connected XInput devices).  We keep the Raw Input pump
//! alive because it gives us:
//!   - device arrival/removal events (WM_INPUT_DEVICE_CHANGE)
//!   - a stable device *handle* that never changes even on controller swap
//!
//! The XInput values are stored under the same handle keys by matching
//! device enumeration order: XInput user index N ↔ Nth gamepad in
//! `enumerate_gamepads()`.  This is imperfect (order can drift) and will be
//! replaced by proper HID-preparsed-data parsing in Phase 2.

use super::GamepadState;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use crate::controller::enumerate::enumerate_gamepads;
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{HANDLE, HWND, LRESULT, WPARAM, LPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::{
                Input::{
                    GetRawInputData, RegisterRawInputDevices, RAWINPUT, RAWINPUTDEVICE,
                    RAWINPUTHEADER, RID_INPUT, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK,
                },
                WindowsAndMessaging::{
                    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
                    PostQuitMessage, RegisterClassW, MSG, WINDOW_EX_STYLE, WM_DESTROY,
                    WM_INPUT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WM_INPUT_DEVICE_CHANGE,
                    CS_HREDRAW, CS_VREDRAW,
                },
            },
        },
    };
    use xinput::XInputHandle;
    use std::{mem, thread, time::Duration};

    const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
    const HID_USAGE_GENERIC_JOYSTICK: u16 = 0x04;
    const HID_USAGE_GENERIC_GAMEPAD: u16 = 0x05;

    /// Spawn the Win32 message-pump thread and the XInput polling thread.
    /// Returns the shared state map.
    pub fn start_capture(
        states: Arc<RwLock<HashMap<u64, GamepadState>>>,
        devices: Arc<RwLock<Vec<crate::controller::PhysicalDevice>>>,
    ) {
        // ---- Raw Input message pump ----
        {
            let states_ri = Arc::clone(&states);
            let devices_ri = Arc::clone(&devices);
            thread::spawn(move || {
                if let Err(e) = run_message_pump(states_ri, devices_ri) {
                    log::error!("Raw Input pump exited with error: {e}");
                }
            });
        }

        // ---- XInput polling thread ----
        // Phase 1: poll XInput user indices 0-3 and map to our device handles
        // by matching the order returned by enumerate_gamepads().
        {
            let states_xi = Arc::clone(&states);
            let devices_xi = Arc::clone(&devices);
            thread::spawn(move || {
                let xi = match XInputHandle::load_default() {
                    Ok(h) => h,
                    Err(e) => {
                        log::error!("Failed to load XInput: {e}");
                        return;
                    }
                };

                loop {
                    // Take a snapshot of device handles in current order
                    let handles: Vec<u64> = {
                        let guard = devices_xi.read().unwrap();
                        guard.iter().map(|d| d.handle).collect()
                    };

                    for (xi_index, &handle) in handles.iter().enumerate().take(4) {
                        let user_index = xi_index as u32;
                        if let Ok(state) = xi.get_state(user_index) {
                            let gp = &state.raw.Gamepad;
                            let gs = GamepadState {
                                buttons: gp.wButtons as u32,
                                left_trigger: gp.bLeftTrigger,
                                right_trigger: gp.bRightTrigger,
                                left_thumb_x: gp.sThumbLX,
                                left_thumb_y: gp.sThumbLY,
                                right_thumb_x: gp.sThumbRX,
                                right_thumb_y: gp.sThumbRY,
                            };
                            let mut map = states_xi.write().unwrap();
                            map.insert(handle, gs);
                        }
                    }

                    thread::sleep(Duration::from_millis(4)); // ~250 Hz
                }
            });
        }
    }

    fn run_message_pump(
        _states: Arc<RwLock<HashMap<u64, GamepadState>>>,
        devices: Arc<RwLock<Vec<crate::controller::PhysicalDevice>>>,
    ) -> anyhow::Result<()> {
        unsafe {
            let hinstance = GetModuleHandleW(None)?;

            // Register window class
            let class_name: Vec<u16> = "BenchBossRawInput\0".encode_utf16().collect();
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);

            // Create hidden message-only window
            let window_name: Vec<u16> = "BenchBossHidden\0".encode_utf16().collect();
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name.as_ptr()),
                PCWSTR(window_name.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                0, 0, 0, 0,
                None,
                None,
                hinstance,
                None,
            )?;

            // Register for Raw Input (gamepads + joysticks), with INPUTSINK + DEVNOTIFY
            let devices_to_register = [
                RAWINPUTDEVICE {
                    usUsagePage: HID_USAGE_PAGE_GENERIC,
                    usUsage: HID_USAGE_GENERIC_GAMEPAD,
                    dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
                    hwndTarget: hwnd,
                },
                RAWINPUTDEVICE {
                    usUsagePage: HID_USAGE_PAGE_GENERIC,
                    usUsage: HID_USAGE_GENERIC_JOYSTICK,
                    dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
                    hwndTarget: hwnd,
                },
            ];

            RegisterRawInputDevices(&devices_to_register, mem::size_of::<RAWINPUTDEVICE>() as u32)
                .ok()?;

            log::info!("Raw Input message pump started");

            // Message loop
            let mut msg: MSG = mem::zeroed();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                // On WM_INPUT_DEVICE_CHANGE we refresh the device list
                if msg.message == WM_INPUT_DEVICE_CHANGE {
                    match enumerate_gamepads() {
                        Ok(new_devs) => {
                            let mut guard = devices.write().unwrap();
                            // Merge: keep existing names, add new handles
                            let existing: HashMap<u64, Option<String>> = guard
                                .iter()
                                .map(|d| (d.handle, d.player_name.clone()))
                                .collect();
                            *guard = new_devs
                                .into_iter()
                                .map(|mut d| {
                                    if let Some(name) = existing.get(&d.handle) {
                                        d.player_name = name.clone();
                                    }
                                    d
                                })
                                .collect();
                        }
                        Err(e) => log::warn!("Re-enumerate failed: {e}"),
                    }
                }
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }

    extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                // WM_INPUT is handled in the message loop above for the device-change path;
                // actual HID data parsing is deferred to Phase 2.
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------
#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;
    use crate::controller::PhysicalDevice;

    pub fn start_capture(
        _states: Arc<RwLock<HashMap<u64, GamepadState>>>,
        _devices: Arc<RwLock<Vec<PhysicalDevice>>>,
    ) {
        log::warn!("start_capture: Raw Input not available on non-Windows — no-op");
    }
}

/// Start the capture threads.  Safe to call only once.
pub fn start_capture(
    states: Arc<RwLock<HashMap<u64, GamepadState>>>,
    devices: Arc<RwLock<Vec<crate::controller::PhysicalDevice>>>,
) {
    imp::start_capture(states, devices);
}
