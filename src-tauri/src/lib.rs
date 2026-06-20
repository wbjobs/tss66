mod config;
mod ring_buffer;
mod monitor;
mod guardian;
mod logger;

use config::{AppConfig, LauncherEntry};
use monitor::ProcessMonitor;
use guardian::{ProcessGuardian, GuardianStatus};
use logger::{Logger, SharedLogger, MetricLog, EventLog};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tauri::{Manager, State};
use ring_buffer::ProcessMetric;

struct AppState {
    monitor: Arc<Mutex<ProcessMonitor>>,
    guardian: Arc<Mutex<ProcessGuardian>>,
    logger: SharedLogger,
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

// ============ Launcher Commands ============

#[tauri::command]
fn add_launcher_entry(entry: LauncherEntry, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };
    config.launcher.push(entry);
    config.save()?;
    {
        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());
    }
    Ok(())
}

#[tauri::command]
fn remove_launcher_entry(label: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };
    config.launcher.retain(|e| e.label != label);
    config.save()?;
    {
        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());
    }
    Ok(())
}

#[tauri::command]
fn update_launcher_entry(entry: LauncherEntry, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };
    if let Some(existing) = config.launcher.iter_mut().find(|e| e.label == entry.label) {
        *existing = entry;
    }
    config.save()?;
    {
        let mut monitor = state.monitor.lock();
        monitor.update_config(config.clone());
    }
    Ok(())
}

#[tauri::command]
fn launch_program(label: String, state: State<'_, AppState>) -> Result<String, String> {
    let config = {
        let monitor = state.monitor.lock();
        monitor.get_config().clone()
    };

    let entry = config
        .launcher
        .iter()
        .find(|e| e.label == label)
        .ok_or_else(|| format!("未找到启动项: {}", label))?;

    let path = Path::new(&entry.path);
    let proc_name_for_log = entry
        .process_name
        .clone()
        .unwrap_or_else(|| {
            path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.path.clone())
        });

    let mut cmd = Command::new(&entry.path);
    cmd.stdin(Stdio::null());
    if let Some(args) = &entry.args {
        cmd.args(args);
    }

    let _ = cmd
        .spawn()
        .map_err(|e| format!("启动程序失败: {}", e))?;

    if entry.auto_monitor {
        let process_name = entry.process_name.clone().unwrap_or_else(|| {
            path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| entry.path.clone())
        });
        let mut cfg = {
            let monitor = state.monitor.lock();
            monitor.get_config().clone()
        };
        if !cfg.process_names.contains(&process_name) {
            cfg.process_names.push(process_name.clone());
            cfg.save()?;
            {
                let mut monitor = state.monitor.lock();
                monitor.update_config(cfg.clone());
            }
            {
                let mut guardian = state.guardian.lock();
                guardian.update_statuses(&cfg.thresholds);
            }
        }
    }

    {
        let mut logger = state.logger.lock();
        logger.log_launched(&proc_name_for_log, &entry.path);
    }

    Ok(format!("已启动: {}", entry.label))
}

// ============ Logger Commands ============

#[tauri::command]
fn get_metric_logs(
    process_name: Option<String>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<MetricLog>, String> {
    let logger = state.logger.lock();
    logger.query_metric_logs(process_name.as_deref(), limit.unwrap_or(200))
}

#[tauri::command]
fn get_event_logs(
    event_type: Option<String>,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<EventLog>, String> {
    let logger = state.logger.lock();
    logger.query_event_logs(event_type.as_deref(), limit.unwrap_or(200))
}

#[tauri::command]
fn cleanup_logs(days_keep: Option<u32>, state: State<'_, AppState>) -> Result<(), String> {
    let mut logger = state.logger.lock();
    logger.cleanup_old_logs(days_keep.unwrap_or(30));
    Ok(())
}

// ============ Main Entry ============

pub fn run() {
    let config = AppConfig::load().unwrap_or_default();

    let monitor = Arc::new(Mutex::new(ProcessMonitor::new(config.clone())));
    let guardian = Arc::new(Mutex::new(ProcessGuardian::new()));
    let logger: SharedLogger = Arc::new(Mutex::new(
        Logger::new().expect("初始化日志数据库失败"),
    ));

    {
        let mut g = guardian.lock();
        g.update_statuses(&config.thresholds);
    }

    // 初始日志清理（保留 30 天）
    {
        let mut l = logger.lock();
        l.cleanup_old_logs(30);
    }

    tauri::Builder::default()
        .manage(AppState {
            monitor: Arc::clone(&monitor),
            guardian: Arc::clone(&guardian),
            logger: Arc::clone(&logger),
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
            add_launcher_entry,
            remove_launcher_entry,
            update_launcher_entry,
            launch_program,
            get_metric_logs,
            get_event_logs,
            cleanup_logs,
        ])
        .setup(move |app| {
            let app_handle = app.handle();
            let monitor = Arc::clone(&monitor);
            let guardian = Arc::clone(&guardian);
            let logger = Arc::clone(&logger);

            std::thread::spawn(move || {
                loop {
                    let loop_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        {
                            let mut mon = monitor.lock();
                            mon.refresh();

                            let metrics = mon.get_latest_metrics();
                            {
                                let mut l = logger.lock();
                                l.insert_metrics(&metrics);
                            }

                            let mut guard = guardian.lock();
                            let killed = guard.check_and_kill(&metrics);

                            let mut l = logger.lock();
                            for info in &killed {
                                l.log_killed(info);
                            }

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
