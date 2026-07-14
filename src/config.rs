use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// CirPath の全設定。
/// メニュー画面から編集され、設定ファイル自体もエディタで直接編集できる。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// ファイルブラウザでファイルを選択した際、自動でエディタ画面へ切り替えるか
    pub auto_open_on_select: bool,
    /// タブ幅(半角スペース換算)
    pub tab_width: usize,
    /// エディタで行番号を表示するか
    pub show_line_numbers: bool,
    /// 保存時に末尾の改行を保証するか
    pub ensure_trailing_newline: bool,
    /// ファイルブラウザで隠しファイル(ドットファイル)を表示するか
    pub show_hidden_files: bool,
    /// ステータスバー等のヒント表示(nano風キー案内)を表示するか
    pub show_key_hints: bool,
    /// 表示言語 ("ja" または "en")
    #[serde(default)]
    pub language: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auto_open_on_select: true,
            tab_width: 4,
            show_line_numbers: true,
            ensure_trailing_newline: true,
            show_hidden_files: false,
            show_key_hints: true,
            language: "ja".to_string(),
        }
    }
}

impl Config {
    /// 設定ファイルの保存場所を返す。
    /// OS標準の設定ディレクトリを優先し、取得できない場合は実行ディレクトリ直下を使う。
    pub fn config_path() -> PathBuf {
        if let Some(dir) = dirs::config_dir() {
            let cirpath_dir = dir.join("cirpath");
            let _ = fs::create_dir_all(&cirpath_dir);
            cirpath_dir.join("config.toml")
        } else {
            PathBuf::from("cirpath_config.toml")
        }
    }

    /// 設定ファイルを読み込む。存在しない/壊れている場合はデフォルト値を返す。
    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => {
                let cfg = Config::default();
                cfg.save();
                cfg
            }
        }
    }

    /// 現在の設定をファイルへ書き出す。
    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = fs::write(path, text);
        }
    }
}
