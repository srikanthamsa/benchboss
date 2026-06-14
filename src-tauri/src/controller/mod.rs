pub mod capture;
pub mod enumerate;
pub mod vigem;

use serde::{Deserialize, Serialize};

/// A physical gamepad detected via Raw Input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalDevice {
    /// Raw Input HANDLE cast to u64 for serialisation.
    pub handle: u64,
    /// HID device path string (e.g. `\\?\HID#VID_...`).
    pub device_path: String,
    /// Human-readable player name assigned after identification.
    pub player_name: Option<String>,
}

/// Normalised gamepad state used for routing.
/// All axes are in the range [-32768, 32767] (matching XInput conventions).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GamepadState {
    pub buttons: u32, // XInput-style bitmask (XINPUT_GAMEPAD_*)
    pub left_trigger: u8,
    pub right_trigger: u8,
    pub left_thumb_x: i16,
    pub left_thumb_y: i16,
    pub right_thumb_x: i16,
    pub right_thumb_y: i16,
}
