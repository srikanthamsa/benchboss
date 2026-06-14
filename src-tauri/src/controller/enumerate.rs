//! Enumerate connected HID gamepad devices via the Windows Raw Input API.

use super::PhysicalDevice;
use anyhow::Result;

// ---------------------------------------------------------------------------
// Windows implementation
// ---------------------------------------------------------------------------
#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use windows::Win32::Devices::HumanInterfaceDevice::{
        HidD_GetProductString, HIDP_CAPS, HidP_GetCaps, HidP_Input,
        HidD_GetPreparsedData, HidD_FreePreparsedData,
    };
    use windows::Win32::UI::Input::{
        GetRawInputDeviceList, GetRawInputDeviceInfoW, RAWINPUTDEVICELIST,
        RIDI_DEVICEINFO, RIDI_DEVICENAME, RID_DEVICE_INFO, RIM_TYPEHID,
    };
    use windows::Win32::Foundation::{HANDLE, BOOLEAN};
    use windows::core::PCWSTR;
    use std::mem;

    const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
    const HID_USAGE_GENERIC_JOYSTICK: u16 = 0x04;
    const HID_USAGE_GENERIC_GAMEPAD: u16 = 0x05;

    pub fn enumerate_gamepads() -> Result<Vec<PhysicalDevice>> {
        let mut num_devices: u32 = 0;
        let device_size = mem::size_of::<RAWINPUTDEVICELIST>() as u32;

        // First call: get count
        unsafe {
            GetRawInputDeviceList(None, &mut num_devices, device_size);
        }

        if num_devices == 0 {
            return Ok(vec![]);
        }

        let mut device_list: Vec<RAWINPUTDEVICELIST> =
            vec![unsafe { mem::zeroed() }; num_devices as usize];

        let count = unsafe {
            GetRawInputDeviceList(
                Some(device_list.as_mut_ptr()),
                &mut num_devices,
                device_size,
            )
        };

        if count == u32::MAX {
            anyhow::bail!("GetRawInputDeviceList failed");
        }

        let mut result = Vec::new();

        for entry in &device_list[..count as usize] {
            if entry.dwType != RIM_TYPEHID {
                continue;
            }

            // Get device info to check usage page/usage
            let mut info: RID_DEVICE_INFO = unsafe { mem::zeroed() };
            info.cbSize = mem::size_of::<RID_DEVICE_INFO>() as u32;
            let mut info_size = mem::size_of::<RID_DEVICE_INFO>() as u32;

            let ret = unsafe {
                GetRawInputDeviceInfoW(
                    HANDLE(entry.hDevice.0),
                    RIDI_DEVICEINFO,
                    Some(&mut info as *mut _ as *mut _),
                    &mut info_size,
                )
            };

            if ret == u32::MAX {
                continue;
            }

            let hid_info = unsafe { &info.Anonymous.hid };

            // Accept generic joystick (0x04) or gamepad (0x05) on the generic page
            let is_gamepad = hid_info.usUsagePage == HID_USAGE_PAGE_GENERIC
                && (hid_info.usUsage == HID_USAGE_GENERIC_JOYSTICK
                    || hid_info.usUsage == HID_USAGE_GENERIC_GAMEPAD);

            if !is_gamepad {
                continue;
            }

            // Get device name/path
            let mut name_len: u32 = 0;
            unsafe {
                GetRawInputDeviceInfoW(
                    HANDLE(entry.hDevice.0),
                    RIDI_DEVICENAME,
                    None,
                    &mut name_len,
                );
            }

            let mut name_buf: Vec<u16> = vec![0u16; name_len as usize];
            unsafe {
                GetRawInputDeviceInfoW(
                    HANDLE(entry.hDevice.0),
                    RIDI_DEVICENAME,
                    Some(name_buf.as_mut_ptr() as *mut _),
                    &mut name_len,
                );
            }

            // Trim null terminator
            while name_buf.last() == Some(&0) {
                name_buf.pop();
            }
            let device_path = String::from_utf16_lossy(&name_buf);

            result.push(PhysicalDevice {
                handle: entry.hDevice.0 as u64,
                device_path,
                player_name: None,
            });
        }

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub
// ---------------------------------------------------------------------------
#[cfg(not(target_os = "windows"))]
mod imp {
    use super::*;

    pub fn enumerate_gamepads() -> Result<Vec<PhysicalDevice>> {
        log::warn!("enumerate_gamepads: not supported on non-Windows — returning empty list");
        Ok(vec![])
    }
}

/// Public entry point — works on all targets.
pub fn enumerate_gamepads() -> Result<Vec<PhysicalDevice>> {
    imp::enumerate_gamepads()
}
