//! Platform abstraction layer for Windows and Linux

pub mod traits;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

pub use traits::*;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as CurrentPlatform;

#[cfg(target_os = "windows")]
pub use windows::windows_api::{
    get_active_explorer_directory, get_active_explorer_or_user_dir, get_startup_lnk_path,
    get_user_home_dir, is_autostart_enabled, set_autostart_enabled,
};

#[cfg(not(target_os = "windows"))]
pub fn get_user_home_dir() -> std::path::PathBuf {
    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_explorer_directory() -> Option<std::path::PathBuf> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_explorer_or_user_dir() -> std::path::PathBuf {
    get_user_home_dir()
}

#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as CurrentPlatform;

#[allow(clippy::default_constructed_unit_structs)]
pub fn get_platform() -> CurrentPlatform {
    CurrentPlatform::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "windows")]
    use ch57x_domain::triggers::{PhantomTrigger, TriggerBank};

    #[test]
    fn test_platform_backend_instantiation() {
        let platform = get_platform();
        let matrix = platform.capability_matrix();
        assert!(!matrix.platform_name.is_empty());
    }

    #[test]
    fn test_capability_matrices() {
        let win_matrix = ch57x_domain::capabilities::CapabilityMatrix::windows_native();
        assert_eq!(win_matrix.platform_name, "Windows 10/11 (Win32)");
        assert!(win_matrix.window_placement.is_supported());
        assert!(win_matrix.window_picker.is_supported());
        assert!(win_matrix.global_hotkeys.is_supported());
        assert!(win_matrix.input_injection.is_supported());

        let x11_matrix = ch57x_domain::capabilities::CapabilityMatrix::linux_x11();
        assert_eq!(x11_matrix.platform_name, "Linux (X11)");
        assert!(x11_matrix.window_placement.is_supported());

        let wayland_matrix = ch57x_domain::capabilities::CapabilityMatrix::linux_wayland();
        assert_eq!(wayland_matrix.platform_name, "Linux (Wayland)");
        assert!(!wayland_matrix.window_placement.is_supported());
        assert!(!wayland_matrix.window_picker.is_supported());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_trigger_conversion_all_banks() {
        use ::windows::Win32::UI::Input::KeyboardAndMouse::{
            MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VIRTUAL_KEY, VK_F1, VK_OEM_MINUS,
        };

        // Bank A: Ctrl+Alt+Shift + F1..F11, 0
        let t_a0 = PhantomTrigger::from_pool_index(0).unwrap();
        let (mods_a0, vk_a0) = crate::windows::hotkeys::trigger_to_win32(&t_a0);
        assert_eq!(mods_a0, MOD_CONTROL | MOD_ALT | MOD_SHIFT);
        assert_eq!(vk_a0, VK_F1);

        let t_a11 = PhantomTrigger::from_pool_index(11).unwrap();
        let (mods_a11, vk_a11) = crate::windows::hotkeys::trigger_to_win32(&t_a11);
        assert_eq!(mods_a11, MOD_CONTROL | MOD_ALT | MOD_SHIFT);
        assert_eq!(vk_a11, VIRTUAL_KEY(0x30)); // '0'

        // Bank B: Ctrl+Alt+Shift + 1..9, a, b, c
        let t_b = PhantomTrigger::from_pool_index(12).unwrap();
        let (mods_b, vk_b) = crate::windows::hotkeys::trigger_to_win32(&t_b);
        assert_eq!(mods_b, MOD_CONTROL | MOD_ALT | MOD_SHIFT);
        assert_eq!(vk_b, VIRTUAL_KEY(0x31)); // '1'

        // Bank C: Ctrl+Alt+Shift + d..o
        let t_c = PhantomTrigger::from_pool_index(24).unwrap();
        let (mods_c, vk_c) = crate::windows::hotkeys::trigger_to_win32(&t_c);
        assert_eq!(mods_c, MOD_CONTROL | MOD_ALT | MOD_SHIFT);
        assert_eq!(vk_c, VIRTUAL_KEY(0x44)); // 'D'

        // Bank D: Ctrl+Alt+Shift + p..z, minus
        let t_d = PhantomTrigger::from_pool_index(47).unwrap();
        let (mods_d, vk_d) = crate::windows::hotkeys::trigger_to_win32(&t_d);
        assert_eq!(mods_d, MOD_CONTROL | MOD_ALT | MOD_SHIFT);
        assert_eq!(vk_d, VK_OEM_MINUS);

        // Bank E: Ctrl+Alt+Win + F1..F11, 0
        let t_e = PhantomTrigger::from_pool_index(48).unwrap();
        let (mods_e, vk_e) = crate::windows::hotkeys::trigger_to_win32(&t_e);
        assert_eq!(mods_e, MOD_CONTROL | MOD_ALT | MOD_WIN);
        assert_eq!(vk_e, VK_F1);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_token_to_virtual_key() {
        use ::windows::Win32::UI::Input::KeyboardAndMouse::{
            VK_F13, VK_F24, VK_LCONTROL, VK_RETURN, VK_SPACE, VK_VOLUME_UP,
        };
        assert_eq!(
            crate::windows::input::token_to_virtual_key("f13"),
            Some(VK_F13)
        );
        assert_eq!(
            crate::windows::input::token_to_virtual_key("F24"),
            Some(VK_F24)
        );
        assert_eq!(
            crate::windows::input::token_to_virtual_key("ctrl"),
            Some(VK_LCONTROL)
        );
        assert_eq!(
            crate::windows::input::token_to_virtual_key("enter"),
            Some(VK_RETURN)
        );
        assert_eq!(
            crate::windows::input::token_to_virtual_key("space"),
            Some(VK_SPACE)
        );
        assert_eq!(
            crate::windows::input::token_to_virtual_key("volumeup"),
            Some(VK_VOLUME_UP)
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_monitors_enumeration() {
        let monitors = crate::windows::monitors::enumerate_monitors();
        assert!(
            !monitors.is_empty(),
            "Should enumerate at least one monitor on Windows"
        );
        let work_area = crate::windows::monitors::get_monitor_work_area(None);
        assert!(work_area.width > 0);
        assert!(work_area.height > 0);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_uipi_check() {
        let _ = crate::windows::input::check_uipi_integrity();
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_autostart_query() {
        let platform = get_platform();
        let enabled = platform.autostart().is_enabled();
        assert!(enabled.is_ok());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_autostart_full_lifecycle() {
        const TEST_KEY: &str = "VortexDeck_Test_Only";

        // 1. Abilita autostart su chiave di test isolata (senza toccare o cancellare l'autostart reale dell'utente!)
        let set_res = crate::windows::windows_api::set_autostart_enabled_named(TEST_KEY, true);
        assert!(
            set_res.is_ok(),
            "set_autostart_enabled_named(true) should succeed: {:?}",
            set_res
        );

        // 2. Verifica presenza chiave registro di test tramite is_autostart_enabled_named
        let enabled = crate::windows::windows_api::is_autostart_enabled_named(TEST_KEY);
        assert!(
            enabled,
            "is_autostart_enabled_named should return true after enable"
        );

        // 3. Disabilita autostart di test
        let unset_res = crate::windows::windows_api::set_autostart_enabled_named(TEST_KEY, false);
        assert!(
            unset_res.is_ok(),
            "set_autostart_enabled_named(false) should succeed: {:?}",
            unset_res
        );

        // 4. Verifica disattivazione
        let disabled = crate::windows::windows_api::is_autostart_enabled_named(TEST_KEY);
        assert!(
            !disabled,
            "is_autostart_enabled_named should return false after disable"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_overlay_invalid_handle() {
        let res = crate::windows::overlay::configure_hud_overlay(0);
        assert!(res.is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_hotkey_registration_lifecycle() {
        let platform = get_platform();
        let triggers = vec![
            PhantomTrigger::new(TriggerBank::BankA, 13).unwrap(),
            PhantomTrigger::new(TriggerBank::BankB, 14).unwrap(),
        ];

        let result = platform.hotkeys().register_triggers(&triggers);
        assert!(result.is_ok(), "Registration should return Ok(conflicts)");

        let unreg_result = platform.hotkeys().unregister_all();
        assert!(unreg_result.is_ok(), "Unregister all should succeed");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_list_visible_windows() {
        let platform = get_platform();
        let windows = platform.windows().list_visible_windows(0);
        assert!(windows.is_ok(), "EnumWindows listing should succeed");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_win32_launch_or_place_window_not_found() {
        use ch57x_domain::actions::{
            PlacementOutcome, WindowCoordinates, WindowPlacement, WindowSelector,
        };

        let platform = get_platform();
        let selector = WindowSelector {
            exe_name: Some("nonexistent_phantom_app_12345.exe".into()),
            title_contains: Some("Nonexistent Window Title 99999".into()),
            class_name: None,
        };
        let placement = WindowPlacement {
            monitor_index: None,
            coordinates: WindowCoordinates::Absolute {
                x: 100,
                y: 100,
                width: 600,
                height: 400,
            },
            bring_to_foreground: false,
        };

        let res = platform
            .windows()
            .launch_or_place_window(&selector, &placement, None, 100);
        assert!(res.is_ok());
        let outcome = res.unwrap();
        assert_eq!(outcome.placement, PlacementOutcome::WindowNotFound);
    }
}
