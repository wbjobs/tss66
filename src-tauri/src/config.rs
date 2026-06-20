use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherEntry {
    pub label: String,
    pub path: String,
    pub args: Option<Vec<String>>,
    pub auto_monitor: bool,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub process_names: Vec<String>,
    pub thresholds: HashMap<String, f64>,
    pub launcher: Vec<LauncherEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            process_names: vec![
                "notepad.exe".to_string(),
                "explorer.exe".to_string(),
            ],
            thresholds: HashMap::new(),
            launcher: vec![
                LauncherEntry {
                    label: "记事本".to_string(),
                    path: "notepad.exe".to_string(),
                    args: None,
                    auto_monitor: true,
                    process_name: Some("notepad.exe".to_string()),
                },
                LauncherEntry {
                    label: "命令行".to_string(),
                    path: "cmd.exe".to_string(),
                    args: None,
                    auto_monitor: true,
                    process_name: Some("cmd.exe".to_string()),
                },
            ],
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;
        if !path.exists() {
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        let mut config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;
        if config.launcher.is_empty() {
            config.launcher = Self::default().launcher;
        }
        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, String> {
        let proj_dirs = directories::ProjectDirs::from("com", "processguardian", "ProcessGuardian")
            .ok_or_else(|| "无法获取配置目录".to_string())?;
        Ok(proj_dirs.config_dir().join("config.json"))
    }

    pub fn data_dir() -> Result<PathBuf, String> {
        let proj_dirs = directories::ProjectDirs::from("com", "processguardian", "ProcessGuardian")
            .ok_or_else(|| "无法获取数据目录".to_string())?;
        let dir = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;
        Ok(dir)
    }
}
