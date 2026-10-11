mod resources;

use auto_launch::{AutoLaunch, AutoLaunchBuilder, LinuxLaunchMode};
use std::path::{Path, PathBuf};
use std::{fmt, io};

const APP_NAME: &str = "Med-Tracker";
const MINIMIZED_ARG: &str = "--minimized";
const INSTALL_RELATIVE_PATH: &str = ".local/opt/med-tracker";

#[derive(Debug)]
pub struct StartupError(String);

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for StartupError {}

impl From<io::Error> for StartupError {
    fn from(error: io::Error) -> Self {
        Self(error.to_string())
    }
}

fn installed_directory() -> Result<PathBuf, StartupError> {
    let home =
        dirs::home_dir().ok_or_else(|| StartupError("Could not find home directory".into()))?;
    let executable = std::env::current_exe()?;
    Ok(autostart_directory_for(
        &executable,
        &home.join(INSTALL_RELATIVE_PATH),
    ))
}

fn autostart_directory_for(executable: &Path, local_bundle: &Path) -> PathBuf {
    if let Some(directory) = executable.parent() {
        if executable.file_name().is_some_and(|name| name == APP_NAME)
            && validate_install(directory).is_ok()
        {
            return directory.to_path_buf();
        }
    }
    local_bundle.to_path_buf()
}

fn registration(app_path: &Path) -> Result<AutoLaunch, StartupError> {
    let path = app_path
        .to_str()
        .ok_or_else(|| StartupError("Installed executable path is not valid UTF-8".into()))?;
    let mut builder = AutoLaunchBuilder::new();
    builder
        .set_app_name(APP_NAME)
        .set_app_path(path)
        .set_args(&[MINIMIZED_ARG])
        .set_linux_launch_mode(LinuxLaunchMode::XdgAutostart);
    builder
        .build()
        .map_err(|error| StartupError(format!("Could not prepare startup registration: {error}")))
}

fn validate_install(directory: &Path) -> Result<PathBuf, StartupError> {
    let executable = directory.join(APP_NAME);
    if !executable.is_file() {
        return Err(StartupError(format!(
            "Installed Med-Tracker executable not found at {}. Run scripts/install-local.sh first.",
            executable.display()
        )));
    }
    for asset_dir in ["icons", "audio"] {
        if !directory.join(asset_dir).is_dir() {
            return Err(StartupError(format!(
                "Installed Med-Tracker bundle is missing {}.",
                directory.join(asset_dir).display()
            )));
        }
    }
    Ok(executable)
}

/// Enables or disables XDG login startup for an installed release bundle.
pub fn set_autostart(enabled: bool) -> Result<(), StartupError> {
    let directory = installed_directory()?;
    let executable = if enabled {
        validate_install(&directory)?
    } else {
        directory.join(APP_NAME)
    };
    let auto = registration(&executable)?;

    if enabled {
        auto.enable()
            .map_err(|error| StartupError(format!("Failed to enable login startup: {error}")))?;
        if !auto
            .is_enabled()
            .map_err(|error| StartupError(format!("Could not verify login startup: {error}")))?
        {
            return Err(StartupError(
                "The startup package did not create the XDG entry.".into(),
            ));
        }
    } else {
        auto.disable()
            .map_err(|error| StartupError(format!("Failed to disable login startup: {error}")))?;
        if auto
            .is_enabled()
            .map_err(|error| StartupError(format!("Could not verify startup removal: {error}")))?
        {
            return Err(StartupError(
                "The XDG startup entry is still present after disabling.".into(),
            ));
        }
    }
    Ok(())
}

pub fn resource_root_for(executable: &Path) -> Result<PathBuf, StartupError> {
    resources::resolve_resource_root(executable).map_err(StartupError)
}

pub fn use_resource_root() -> Result<(), StartupError> {
    let executable = std::env::current_exe()?;
    let root = resource_root_for(&executable)?;
    std::env::set_current_dir(root)?;
    Ok(())
}

pub fn apply_startup_preference(
    previous: bool,
    requested: bool,
    mut register: impl FnMut(bool) -> Result<(), String>,
    mut persist: impl FnMut(bool) -> Result<(), String>,
) -> Result<(), String> {
    if let Err(update_error) = register(requested) {
        return match register(previous) {
            Ok(()) => Err(format!(
                "Could not update login startup: {update_error}. The previous startup setting was restored."
            )),
            Err(rollback_error) => Err(format!(
                "Could not update login startup: {update_error}. Restoring the previous setting also failed: {rollback_error}."
            )),
        };
    }
    if let Err(save_error) = persist(requested) {
        return match register(previous) {
            Ok(()) => Err(format!(
                "Could not save settings: {save_error}. The startup change was reverted."
            )),
            Err(rollback_error) => Err(format!(
                "Could not save settings: {save_error}. Startup rollback also failed: {rollback_error}."
            )),
        };
    }
    Ok(())
}

pub fn is_started_minimized() -> bool {
    std::env::args().skip(1).any(|arg| arg == MINIMIZED_ARG)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupWindow {
    Open,
    Hidden,
    WaitForTray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayStartupEvent {
    Ready,
    Failed,
    Timeout,
    UserActivated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrayStartupTransition {
    pub waiting_for_tray: bool,
    pub tray_ready: bool,
    pub show_window: bool,
}

pub fn tray_startup_transition(
    waiting_for_tray: bool,
    tray_ready: bool,
    window_open: bool,
    event: TrayStartupEvent,
) -> TrayStartupTransition {
    match event {
        TrayStartupEvent::Ready => TrayStartupTransition {
            waiting_for_tray: false,
            tray_ready: true,
            show_window: false,
        },
        TrayStartupEvent::Failed => TrayStartupTransition {
            waiting_for_tray: false,
            tray_ready: false,
            show_window: !window_open,
        },
        TrayStartupEvent::Timeout if waiting_for_tray && !tray_ready => TrayStartupTransition {
            waiting_for_tray: false,
            tray_ready,
            show_window: !window_open,
        },
        TrayStartupEvent::UserActivated => TrayStartupTransition {
            waiting_for_tray: false,
            tray_ready,
            show_window: !window_open,
        },
        TrayStartupEvent::Timeout => TrayStartupTransition {
            waiting_for_tray,
            tray_ready,
            show_window: false,
        },
    }
}

pub fn startup_window_decision(
    started_minimized: bool,
    tray_enabled: bool,
    wait_for_tray: bool,
    tray_ready: bool,
    resources_ready: bool,
) -> StartupWindow {
    if !started_minimized || !tray_enabled || !resources_ready {
        StartupWindow::Open
    } else if wait_for_tray && !tray_ready {
        StartupWindow::WaitForTray
    } else if tray_ready {
        StartupWindow::Hidden
    } else {
        StartupWindow::Open
    }
}

pub fn tray_loss_fallback_due(
    tray_enabled: bool,
    embedded: bool,
    window_hidden: bool,
    timer_generation: u64,
    current_generation: u64,
) -> bool {
    tray_enabled && !embedded && window_hidden && timer_generation == current_generation
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn packaged_executable_is_used_for_autostart_instead_of_local_bundle() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("med-tracker-package-{stamp}"));
        let bundle = root.join("usr/lib/med-tracker");
        fs::create_dir_all(bundle.join("icons")).unwrap();
        fs::create_dir_all(bundle.join("audio")).unwrap();
        let executable = bundle.join(APP_NAME);
        fs::write(&executable, "test executable").unwrap();
        let local = root.join("home/.local/opt/med-tracker");
        assert_eq!(autostart_directory_for(&executable, &local), bundle);
        assert_eq!(
            autostart_directory_for(&root.join("target/release/Med-Tracker"), &local),
            local
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registration_failure_keeps_the_previous_preference() {
        let enabled = false;
        let mut applied = Vec::new();
        let result = apply_startup_preference(
            enabled,
            true,
            |requested| {
                applied.push(requested);
                if requested {
                    Err("registration failed".to_string())
                } else {
                    Ok(())
                }
            },
            |_| panic!("settings should not be saved after registration fails"),
        );

        assert!(result.is_err());
        assert_eq!(applied, vec![true, false]);
        assert!(!enabled);
    }

    #[test]
    fn registration_and_rollback_failures_are_both_reported() {
        let mut calls = 0;
        let result = apply_startup_preference(
            false,
            true,
            |_| {
                calls += 1;
                Err(if calls == 1 {
                    "enable failed".to_string()
                } else {
                    "disable rollback failed".to_string()
                })
            },
            |_| panic!("settings should not be saved after registration fails"),
        );

        let error = result.unwrap_err();
        assert!(error.contains("enable failed"));
        assert!(error.contains("disable rollback failed"));
    }

    #[test]
    fn persistence_failure_rolls_back_registration_and_preference() {
        let enabled = false;
        let mut applied = Vec::new();
        let result = apply_startup_preference(
            enabled,
            true,
            |value| {
                applied.push(value);
                Ok(())
            },
            |_| Err("settings disk full".to_string()),
        );

        assert!(result.unwrap_err().contains("settings disk full"));
        assert_eq!(applied, vec![true, false]);
        assert!(!enabled);
    }

    #[test]
    fn rollback_failure_is_included_in_the_reported_error() {
        let enabled = false;
        let mut calls = 0;
        let result = apply_startup_preference(
            enabled,
            true,
            |_| {
                calls += 1;
                if calls == 1 {
                    Ok(())
                } else {
                    Err("rollback failed".to_string())
                }
            },
            |_| Err("settings disk full".to_string()),
        );

        let error = result.unwrap_err();
        assert!(error.contains("settings disk full"));
        assert!(error.contains("rollback failed"));
        assert!(!enabled);
    }

    #[test]
    fn only_waits_or_hides_when_a_minimized_start_has_a_usable_tray() {
        assert_eq!(
            startup_window_decision(false, true, false, false, true),
            StartupWindow::Open
        );
        assert_eq!(
            startup_window_decision(true, true, false, true, true),
            StartupWindow::Hidden
        );
        assert_eq!(
            startup_window_decision(true, true, false, false, true),
            StartupWindow::Open
        );
        assert_eq!(
            startup_window_decision(true, true, true, false, true),
            StartupWindow::WaitForTray
        );
        assert_eq!(
            startup_window_decision(true, false, false, true, true),
            StartupWindow::Open
        );
        assert_eq!(
            startup_window_decision(true, true, false, true, false),
            StartupWindow::Open
        );
    }

    #[test]
    fn hidden_startup_falls_back_when_tray_fails_or_times_out() {
        for event in [TrayStartupEvent::Failed, TrayStartupEvent::Timeout] {
            assert_eq!(
                tray_startup_transition(true, false, false, event),
                TrayStartupTransition {
                    waiting_for_tray: false,
                    tray_ready: false,
                    show_window: true,
                }
            );
        }
    }

    #[test]
    fn late_tray_readiness_does_not_hide_a_window_opened_by_timeout_or_user() {
        for event in [TrayStartupEvent::Timeout, TrayStartupEvent::UserActivated] {
            let transition = tray_startup_transition(true, false, false, event);
            assert!(transition.show_window);
            assert!(!transition.waiting_for_tray);
            assert!(
                !tray_startup_transition(
                    transition.waiting_for_tray,
                    transition.tray_ready,
                    true,
                    TrayStartupEvent::Ready,
                )
                .show_window
            );
        }
    }

    #[test]
    fn tray_failure_reopens_a_hidden_window_but_does_not_duplicate_an_open_one() {
        assert!(tray_startup_transition(false, true, false, TrayStartupEvent::Failed).show_window);
        assert!(!tray_startup_transition(true, false, true, TrayStartupEvent::Failed).show_window);
    }

    #[test]
    fn delayed_tray_loss_fallback_ignores_recovered_or_disabled_generations() {
        assert!(tray_loss_fallback_due(true, false, true, 7, 7));
        assert!(!tray_loss_fallback_due(true, true, true, 7, 8));
        assert!(!tray_loss_fallback_due(false, false, true, 7, 7));
        assert!(!tray_loss_fallback_due(true, false, false, 7, 7));
        assert!(!tray_loss_fallback_due(true, false, true, 7, 8));
    }

    #[test]
    fn registration_round_trip_child() {
        if std::env::var_os("MED_TRACKER_STARTUP_TEST_CHILD").is_none() {
            return;
        }
        let home = dirs::home_dir().unwrap();
        let install = home.join(INSTALL_RELATIVE_PATH);
        fs::create_dir_all(install.join("icons")).unwrap();
        fs::create_dir_all(install.join("audio")).unwrap();
        fs::write(install.join(APP_NAME), "test executable").unwrap();

        set_autostart(true).unwrap();
        set_autostart(true).unwrap();
        assert!(
            registration(&install.join(APP_NAME))
                .unwrap()
                .is_enabled()
                .unwrap()
        );
        let entry = home.join(".config/autostart/Med-Tracker.desktop");
        let contents = fs::read_to_string(&entry).unwrap();
        assert!(contents.contains(&format!(
            "Exec={} --minimized",
            install.join(APP_NAME).display()
        )));

        set_autostart(false).unwrap();
        assert!(!entry.exists(), "saved false removes a stale entry");
        set_autostart(false).unwrap();
        set_autostart(true).unwrap();
        assert!(entry.exists(), "saved true repairs a missing entry");
        fs::remove_file(install.join(APP_NAME)).unwrap();
        assert!(set_autostart(true).is_err());
        set_autostart(false).unwrap();
        assert!(
            !registration(&install.join(APP_NAME))
                .unwrap()
                .is_enabled()
                .unwrap()
        );
    }

    #[test]
    fn package_registration_isolated_home_round_trip() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let home = std::env::temp_dir().join(format!("med-tracker-autostart-{stamp}"));
        fs::create_dir_all(&home).unwrap();
        let result = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "startup::tests::registration_round_trip_child",
                "--nocapture",
            ])
            .env("HOME", &home)
            .env("MED_TRACKER_STARTUP_TEST_CHILD", "1")
            .output()
            .unwrap();
        let _ = fs::remove_dir_all(&home);

        assert!(
            result.status.success(),
            "isolated startup test failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}
