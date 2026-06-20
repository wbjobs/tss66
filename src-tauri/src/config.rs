use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub process_names: Vec<String>,
    pub thresholds: HashMap<String, f64>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            process_names: vec![
                "notepad.exe".to_string(),
                "explorer.exe".to_string(),
            ],
            thresholds: HashMap::new(),
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
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;
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
}
