//! ViGEm virtual Xbox 360 controller management.
//!
//! Creates 4 permanent virtual Xbox 360 controllers on startup and keeps
//! them alive for the duration of the process.  Callers push gamepad state
//! via `update_slot`.

use super::GamepadState;
use anyhow::Result;

// ---------------------------------------------------------------------------
// Windows implementation
// ---------------------------------------------------------------------------
#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use vigem_client::{Client, Xbox360Wired, TargetId, XButtons, XGamepad};

    const SLOT_COUNT: usize = 4;

    pub struct ViGEmTargets {
        // Keep the client alive (owns the bus connection)
        _client: Client,
        targets: Vec<Xbox360Wired<Client>>,
    }

    impl ViGEmTargets {
        pub fn new() -> Result<Self> {
            let client = Client::connect()?;
            log::info!("Connected to ViGEmBus");

            let mut targets = Vec::with_capacity(SLOT_COUNT);
            for i in 0..SLOT_COUNT {
                let mut target = Xbox360Wired::new(client.clone(), TargetId::XBOX360_WIRED);
                target.plugin()?;
                target.wait_ready()?;
                log::info!("Virtual Xbox controller slot {} online", i + 1);
                targets.push(target);
            }

            Ok(Self {
                _client: client,
                targets,
            })
        }

        pub fn update_slot(&mut self, slot: usize, state: &GamepadState) -> Result<()> {
            if slot >= SLOT_COUNT {
                anyhow::bail!("Invalid slot index {slot}");
            }

            let gamepad = XGamepad {
                buttons: XButtons(state.buttons as u16),
                left_trigger: state.left_trigger,
                right_trigger: state.right_trigger,
                thumb_lx: state.left_thumb_x,
                thumb_ly: state.left_thumb_y,
                thumb_rx: state.right_thumb_x,
                thumb_ry: state.right_thumb_y,
            };

            self.targets[slot].update(&gamepad)?;
            Ok(())
        }

        pub fn clear_slot(&mut self, slot: usize) -> Result<()> {
            if slot >= SLOT_COUNT {
                anyhow::bail!("Invalid slot index {slot}");
            }
            self.targets[slot].update(&XGamepad::default())?;
            Ok(())
        }
    }

    pub fn create_targets() -> Result<ViGEmTargets> {
        ViGEmTargets::new()
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------
#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub struct ViGEmTargets;

    impl ViGEmTargets {
        pub fn update_slot(&mut self, slot: usize, _state: &GamepadState) -> Result<()> {
            log::warn!("update_slot({slot}): ViGEm not available on non-Windows — no-op");
            Ok(())
        }

        pub fn clear_slot(&mut self, slot: usize) -> Result<()> {
            log::warn!("clear_slot({slot}): ViGEm not available on non-Windows — no-op");
            Ok(())
        }
    }

    pub fn create_targets() -> Result<ViGEmTargets> {
        log::warn!("create_targets: ViGEm not available on non-Windows — returning stub");
        Ok(ViGEmTargets)
    }
}

// Re-export
pub use imp::{create_targets, ViGEmTargets};
