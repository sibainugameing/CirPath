use std::fs;
use std::path::{Path, PathBuf};

/// 入力プロンプトの種類 (nanoのCtrl+W検索やファイル名入力のような下部プロンプト)
#[derive(Debug, Clone, PartialEq)]
pub enum Prompt {
    None,
    SaveAs(String),
    Search(String),
}

pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub file_path: Option<PathBuf>,
    pub dirty: bool,
    pub status_message: String,
    pub tab_width: usize,
    pub show_line_numbers: bool,
    pub prompt: Prompt,
    pub cut_buffer: Option<String>,
    pub last_search: String,
}

impl Editor {
    pub fn new(tab_width: usize, show_line_numbers: bool) -> Self {
        Editor {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            file_path: None,
            dirty: false,
            status_message: "Ctrl+S 保存 | Ctrl+O 開く | Ctrl+W 検索 | Ctrl+K 行カット | Ctrl+Q/E ウィンドウ切替".to_string(),
            tab_width,
            show_line_numbers,
            prompt: Prompt::None,
            cut_buffer: None,
            last_search: String::new(),
        }
    }

    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).unwrap_or_default();
        self.lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|l| l.to_string()).collect()
        };
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.file_path = Some(path.to_path_buf());
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_row = 0;
        self.dirty = false;
        self.status_message = format!("開きました: {}", path.display());
        Ok(())
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = self.file_path.clone() {
            self.save_to(&path)
        } else {
            self.prompt = Prompt::SaveAs(String::new());
            Ok(())
        }
    }

    pub fn save_to<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let text = self.lines.join("\n");
        fs::write(path, text)?;
        self.file_path = Some(path.to_path_buf());
        self.dirty = false;
        self.status_message = format!("保存しました: {}", path.display());
        Ok(())
    }

    fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor_row)
            .map(|l| l.chars().count())
            .unwrap_or(0)
    }

    pub fn insert_char(&mut self, c: char) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let line = &mut self.lines[row];
        let byte_idx = char_to_byte(line, col);
        line.insert(byte_idx, c);
        self.cursor_col += 1;
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let line = self.lines[row].clone();
        let byte_idx = char_to_byte(&line, col);
        let (left, right) = line.split_at(byte_idx);
        self.lines[row] = left.to_string();
        self.lines.insert(row + 1, right.to_string());
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let row = self.cursor_row;
            let col = self.cursor_col;
            let line = &mut self.lines[row];
            let start = char_to_byte(line, col - 1);
            let end = char_to_byte(line, col);
            line.replace_range(start..end, "");
            self.cursor_col -= 1;
            self.dirty = true;
        } else if self.cursor_row > 0 {
            let cur_line = self.lines.remove(self.cursor_row);
            let prev_len = self.lines[self.cursor_row - 1].chars().count();
            self.lines[self.cursor_row - 1].push_str(&cur_line);
            self.cursor_row -= 1;
            self.cursor_col = prev_len;
            self.dirty = true;
        }
    }

    pub fn delete_forward(&mut self) {
        let len = self.current_line_len();
        if self.cursor_col < len {
            let row = self.cursor_row;
            let col = self.cursor_col;
            let line = &mut self.lines[row];
            let start = char_to_byte(line, col);
            let end = char_to_byte(line, col + 1);
            line.replace_range(start..end, "");
            self.dirty = true;
        } else if self.cursor_row + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
            self.dirty = true;
        }
    }

    pub fn insert_tab(&mut self) {
        for _ in 0..self.tab_width {
            self.insert_char(' ');
        }
    }

    pub fn cut_line(&mut self) {
        if self.lines.len() == 1 {
            self.cut_buffer = Some(self.lines[0].clone());
            self.lines[0].clear();
        } else {
            let removed = self.lines.remove(self.cursor_row);
            self.cut_buffer = Some(removed);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
        }
        self.cursor_col = 0;
        self.dirty = true;
        self.status_message = "行をカットしました (Ctrl+U で貼り付け)".to_string();
    }

    pub fn uncut(&mut self) {
        if let Some(text) = self.cut_buffer.clone() {
            self.lines.insert(self.cursor_row, text);
            self.dirty = true;
            self.status_message = "貼り付けました".to_string();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
        }
    }

    pub fn move_right(&mut self) {
        let len = self.current_line_len();
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.current_line_len());
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.current_line_len());
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.current_line_len();
    }

    pub fn page_up(&mut self, page: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(page);
        self.cursor_col = self.cursor_col.min(self.current_line_len());
    }

    pub fn page_down(&mut self, page: usize) {
        self.cursor_row = (self.cursor_row + page).min(self.lines.len() - 1);
        self.cursor_col = self.cursor_col.min(self.current_line_len());
    }

    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + viewport_height {
            self.scroll_row = self.cursor_row + 1 - viewport_height;
        }
    }

    pub fn search_next(&mut self, query: &str) {
        if query.is_empty() {
            return;
        }
        self.last_search = query.to_string();
        let total = self.lines.len();
        for offset in 1..=total {
            let row = (self.cursor_row + offset) % total;
            if let Some(idx) = self.lines[row].find(query) {
                self.cursor_row = row;
                self.cursor_col = self.lines[row][..idx].chars().count();
                self.status_message = format!("見つかりました: {}行目", row + 1);
                return;
            }
        }
        self.status_message = format!("見つかりませんでした: {}", query);
    }
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}
