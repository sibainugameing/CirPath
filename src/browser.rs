use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowserPrompt {
    None,
    GotoPath(String),
}

pub struct Browser {
    pub current_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub scroll: usize,
    pub status_message: String,
    pub prompt: BrowserPrompt,
}

impl Browser {
    pub fn new(start_dir: PathBuf) -> Self {
        let mut b = Browser {
            current_dir: start_dir,
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            status_message: "Enter 開く | Backspace 上へ | g パス指定 | Ctrl+Q/E 切替".to_string(),
            prompt: BrowserPrompt::None,
        };
        b.refresh();
        b
    }

    pub fn refresh(&mut self) {
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            for e in read_dir.flatten() {
                let path = e.path();
                let name = e.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                entries.push(Entry { name, path, is_dir });
            }
        }
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        self.entries = entries;
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn selected_entry(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    pub fn go_top(&mut self) {
        self.selected = 0;
    }

    pub fn go_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }
    }

    /// 選択中のディレクトリに入る。ファイルの場合はそのパスを返す
    pub fn enter_selected(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.selected_entry().cloned() {
            if entry.is_dir {
                self.current_dir = entry.path;
                self.refresh();
                None
            } else {
                Some(entry.path)
            }
        } else {
            None
        }
    }

    pub fn go_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let prev = self.current_dir.clone();
            self.current_dir = parent.to_path_buf();
            self.refresh();
            if let Some(idx) = self.entries.iter().position(|e| e.path == prev) {
                self.selected = idx;
            }
        }
    }

    /// 絶対パス・相対パスどちらでも指定ディレクトリへ移動
    pub fn goto_path(&mut self, input: &str) {
        let target = Path::new(input);
        let resolved = if target.is_absolute() {
            target.to_path_buf()
        } else {
            self.current_dir.join(target)
        };
        match fs::canonicalize(&resolved) {
            Ok(canon) if canon.is_dir() => {
                self.current_dir = canon;
                self.refresh();
                self.status_message = "移動しました".to_string();
            }
            Ok(canon) if canon.is_file() => {
                self.status_message = "ディレクトリではありません".to_string();
            }
            _ => {
                self.status_message = format!("パスが見つかりません: {}", input);
            }
        }
    }

    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + viewport_height {
            self.scroll = self.selected + 1 - viewport_height;
        }
    }
}
