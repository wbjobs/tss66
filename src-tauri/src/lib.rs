mod config;
mod ring_buffer;
mod monitor;
mod guardian;

use config::AppConfig;
use monitor::ProcessMonitor;
use guardian::{ProcessGuardian, GuardianStatus};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Manager, State};
use ring_buffer::ProcessMetric;

struct AppState {
    monitor: Arc<Mutex<ProcessMonitor>>,
    guardian: Arc<Mutex<ProcessGuardian>>,
}

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let monitor = state.monitor.lock();
    Ok(monitor.get_config().clone())
}

#[tauri::command]
fn update_config(config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    config.save()?;
    {
        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());
    }
    {
        let mut guardian = state.guardian.lock();
        guardian.update_statuses(&config.thresholds);
    }
    Ok(())
}

#[tauri::command]
fn add_process(process_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };

    if !config.process_names.contains(&process_name) {
        config.process_names.push(process_name.clone());
        config.save()?;

        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());

        let mut guardian = state.guardian.lock();
        guardian.update_statuses(&config.thresholds);
    }

    Ok(())
}

#[tauri::command]
fn remove_process(process_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };

    config.process_names.retain(|n| n != &process_name);
    config.thresholds.remove(&process_name);
    config.save()?;

    {
        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());
    }
    {
        let mut guardian = state.guardian.lock();
        guardian.update_statuses(&config.thresholds);
    }

    Ok(())
}

#[tauri::command]
fn set_memory_threshold(
    process_name: String,
    threshold_mb: Option<f64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };

    match threshold_mb {
        Some(mb) if mb > 0.0 => {
            config.thresholds.insert(process_name.clone(), mb);
        }
        _ => {
            config.thresholds.remove(&process_name);
        }
    }

    config.save()?;

    {
        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());
    }
    {
        let mut guardian = state.guardian.lock();
        guardian.update_statuses(&config.thresholds);
    }

    Ok(())
}

#[tauri::command]
fn get_process_data(state: State<'_, AppState>) -> Result<Vec<ProcessMetric>, String> {
    let monitor = state.monitor.lock();
    Ok(monitor.get_latest_metrics())
}

#[tauri::command]
fn get_history_data(state: State<'_, AppState>) -> Result<HashMap<String, Vec<ProcessMetric>>, String> {
    let monitor = state.monitor.lock();
    Ok(monitor.get_history())
}

#[tauri::command]
fn get_guardian_status(state: State<'_, AppState>) -> Result<HashMap<String, GuardianStatus>, String> {
    let guardian = state.guardian.lock();
    Ok(guardian.get_statuses())
}

#[tauri::command]
fn reset_guardian_status(process_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut guardian = state.guardian.lock();
    guardian.reset_status(&process_name);
    Ok(())
}

pub fn run() {
    let config = AppConfig::load().unwrap_or_default();

    let monitor = Arc::new(Mutex::new(ProcessMonitor::new(config.clone())));
    let guardian = Arc::new(Mutex::new(ProcessGuardian::new()));

    {
        let mut g = guardian.lock();
        g.update_statuses(&config.thresholds);
    }

    tauri::Builder::default()
        .manage(AppState {
            monitor: Arc::clone(&monitor),
            guardian: Arc::clone(&guardian),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            add_process,
            remove_process,
            set_memory_threshold,
            get_process_data,
            get_history_data,
            get_guardian_status,
            reset_guardian_status,
        ])
        .setup(move |app| {
            let app_handle = app.handle();
            let monitor = Arc::clone(&monitor);
            let guardian = Arc::clone(&guardian);

            std::thread::spawn(move || {
                loop {
                    let loop_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        {
                            let mut mon = monitor.lock();
                            mon.refresh();

                            let metrics = mon.get_latest_metrics();
                            let mut guard = guardian.lock();
                            let killed = guard.check_and_kill(&metrics);

                            for info in killed {
                                let _ = tauri::api::notification::Notification::new(
                                    "进程守护提醒"
                                )
                                .body(format!(
                                    "进程 {} 内存占用 {:.2}MB 超过阈值 {:.2}MB，已被终止",
                                    info.name, info.memory_mb, info.threshold_mb
                                ))
                                .show();

                                let payload = serde_json::json!({
                                    "process_name": info.name,
                                    "memory_mb": info.memory_mb,
                                    "threshold_mb": info.threshold_mb,
                                });
                                let _ = app_handle.emit_all("process-killed", payload);
                            }
                        }
                    }));

                    if let Err(e) = loop_result {
                        eprintln!("监控线程本轮执行发生异常: {:?}", e);
                    }

                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
