use crate::ring_buffer::ProcessMetric;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KilledProcessInfo {
    pub name: String,
    pub memory_mb: f64,
    pub threshold_mb: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianStatus {
    pub active: bool,
    pub killed: bool,
    pub last_kill_time: Option<u64>,
}

impl Default for GuardianStatus {
    fn default() -> Self {
        Self {
            active: false,
            killed: false,
            last_kill_time: None,
        }
    }
}

pub struct ProcessGuardian {
    statuses: HashMap<String, GuardianStatus>,
    killed_processes: HashMap<String, u64>,
}

impl ProcessGuardian {
    pub fn new() -> Self {
        Self {
            statuses: HashMap::new(),
            killed_processes: HashMap::new(),
        }
    }

    pub fn update_statuses(&mut self, thresholds: &HashMap<String, f64>) {
        for name in thresholds.keys() {
            if !self.statuses.contains_key(name) {
                self.statuses.insert(name.clone(), GuardianStatus {
                    active: true,
                    killed: false,
                    last_kill_time: None,
                });
            } else if let Some(status) = self.statuses.get_mut(name) {
                status.active = true;
            }
        }

        self.statuses.retain(|name, _| thresholds.contains_key(name));
    }

    pub fn check_and_kill(
        &mut self,
        metrics: &[ProcessMetric],
    ) -> Vec<KilledProcessInfo> {
        let mut killed = Vec::new();

        for metric in metrics {
            if let Some(threshold) = metric.threshold_mb {
                if metric.running && metric.memory_mb > threshold {
                    let status = self.statuses.get_mut(&metric.name);
                    let should_kill = match status {
                        Some(s) => !s.killed,
                        None => true,
                    };

                    if should_kill {
                        match kill_process(&metric.name) {
                            Ok(_) => {
                                if let Some(s) = self.statuses.get_mut(&metric.name) {
                                    s.killed = true;
                                    s.last_kill_time = Some(metric.timestamp);
                                }
                                self.killed_processes.insert(
                                    metric.name.clone(),
                                    metric.timestamp,
                                );
                                killed.push(KilledProcessInfo {
                                    name: metric.name.clone(),
                                    memory_mb: metric.memory_mb,
                                    threshold_mb: threshold,
                                    timestamp: metric.timestamp,
                                });
                            }
                            Err(e) => {
                                eprintln!("终止进程 {} 失败: {}", metric.name, e);
                            }
                        }
                    }
                }
            }
        }

        killed
    }

    pub fn get_statuses(&self) -> HashMap<String, GuardianStatus> {
        self.statuses.clone()
    }

    pub fn reset_status(&mut self, process_name: &str) {
        if let Some(status) = self.statuses.get_mut(process_name) {
            status.killed = false;
        }
    }
}

impl Default for ProcessGuardian {
    fn default() -> Self {
        Self::new()
    }
}

fn kill_process(process_name: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("taskkill")
            .args(["/F", "/IM", process_name])
            .output()
            .map_err(|e| format!("执行 taskkill 失败: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("taskkill 执行失败: {}", stderr))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("pkill")
            .args(["-f", process_name])
            .output()
            .map_err(|e| format!("执行 pkill 失败: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("pkill 执行失败: {}", stderr))
        }
    }
}
