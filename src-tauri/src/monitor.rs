use crate::ring_buffer::{ProcessMetric};
use crate::config::AppConfig;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

pub type SharedProcessData = Arc<Mutex<HashMap<String, Vec<ProcessMetric>>>>;

const BUFFER_CAPACITY: usize = 60;

pub struct ProcessMonitor {
    system: System,
    history: HashMap<String, crate::ring_buffer::RingBuffer<ProcessMetric>>,
    config: AppConfig,
}

impl ProcessMonitor {
    pub fn new(config: AppConfig) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let mut history = HashMap::new();
        for name in &config.process_names {
            history.insert(
                name.clone(),
                crate::ring_buffer::RingBuffer::new(BUFFER_CAPACITY),
            );
        }

        Self {
            system,
            history,
            config,
        }
    }

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config.clone();

        for name in &self.config.process_names {
            if !self.history.contains_key(name) {
                self.history.insert(
                    name.clone(),
                    crate::ring_buffer::RingBuffer::new(BUFFER_CAPACITY),
                );
            }
        }

        self.history.retain(|name, _| self.config.process_names.contains(name));
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for proc_name in &self.config.process_names {
            let metric = self.collect_metric(proc_name, timestamp);
            if let Some(buffer) = self.history.get_mut(proc_name) {
                buffer.push(metric);
            }
        }
    }

    fn collect_metric(&self, process_name: &str, timestamp: u64) -> ProcessMetric {
        let name_lower = process_name.to_lowercase();

        let mut total_cpu = 0.0;
        let mut total_memory = 0u64;
        let mut count = 0u32;
        let mut pid = None;

        for (pid_val, process) in self.system.processes() {
            let proc_name = process.name().to_lowercase();
            if proc_name == name_lower || proc_name.starts_with(&name_lower) {
                total_cpu += process.cpu_usage() as f64;
                total_memory += process.memory();
                count += 1;
                if pid.is_none() {
                    pid = Some(pid_val.as_u32());
                }
            }
        }

        let running = count > 0;
        let memory_mb = if running {
            (total_memory as f64) / (1024.0 * 1024.0)
        } else {
            0.0
        };

        let threshold_mb = self.config.thresholds.get(process_name).copied();

        ProcessMetric {
            name: process_name.to_string(),
            pid,
            running,
            cpu_usage: if running { total_cpu } else { 0.0 },
            memory_mb,
            threshold_mb,
            timestamp,
        }
    }

    pub fn get_latest_metrics(&self) -> Vec<ProcessMetric> {
        let mut result = Vec::new();
        for name in &self.config.process_names {
            if let Some(buffer) = self.history.get(name) {
                if let Some(metric) = buffer.latest() {
                    result.push(metric.clone());
                } else {
                    result.push(ProcessMetric {
                        name: name.clone(),
                        pid: None,
                        running: false,
                        cpu_usage: 0.0,
                        memory_mb: 0.0,
                        threshold_mb: self.config.thresholds.get(name).copied(),
                        timestamp: 0,
                    });
                }
            }
        }
        result
    }

    pub fn get_history(&self) -> HashMap<String, Vec<ProcessMetric>> {
        let mut result = HashMap::new();
        for (name, buffer) in &self.history {
            result.insert(name.clone(), buffer.to_vec());
        }
        result
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}
