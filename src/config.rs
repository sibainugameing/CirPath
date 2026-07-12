use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// ファイルブラウザでファイルを選択した際に自動でエディタへ切り替えるか
    pub auto_jump_to_editor: bool,
    /// タブ幅
    pub tab_width: usize,
    /// 行番号を表示するか
    pub show_line_numbers: bool,
    /// 起動時に開くディレクトリ (空なら実行時のカレントディレクトリ)
    pub start_dir: String,
    /// テーマ (dark / light)
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_jump_to_editor: true,
            tab_width: 4,
            show_line_numbers: true,
            start_dir: String::new(),
            theme: "dark".to_string(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("cirpath")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Config {
        let path = Self::config_path();
        if let Ok(text) = fs::read_to_string(&path) {
            match toml::from_str::<Config>(&text) {
                Ok(cfg) => return cfg,
                Err(_) => return Config::default(),
            }
        }
        let cfg = Config::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let text = toml::to_string_pretty(self).unwrap_or_default();
        fs::write(Self::config_path(), text)
    }

    /// 設定ファイルが存在しなければ作成し、パスを返す (エディタで開く用)
    pub fn ensure_and_path() -> PathBuf {
        let path = Self::config_path();
        if !path.exists() {
            let cfg = Config::default();
            let _ = cfg.save();
        }
        path
    }
}
