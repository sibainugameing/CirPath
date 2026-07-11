use crate::error::{CirPathError, CirResult};
use std::fs;
use std::io::Write;

/// テキストの実体を保持する構造体。
/// 行単位（Vec<String>）で管理し、1-indexed（人間にとって自然な行番号）で
/// 外部からアクセスされることを前提とする。
pub struct TextBuffer {
    pub lines: Vec<String>,
    pub filename: Option<String>,
    pub dirty: bool,
    /// Undo用に直前のバッファ全体スナップショットを1世代分保持する。
    undo_snapshot: Option<Vec<String>>,
}

impl TextBuffer {
    pub fn new() -> Self {
        TextBuffer {
            lines: Vec::new(),
            filename: None,
            dirty: false,
            undo_snapshot: None,
        }
    }

    /// ファイルを読み込んでバッファを初期化する。
    pub fn load(&mut self, path: &str) -> CirResult<()> {
        let content = fs::read_to_string(path).map_err(|e| CirPathError::Io {
            path: path.to_string(),
            source: e,
        })?;

        // 改行コードの違いを吸収しつつ行分割する。
        let lines: Vec<String> = if content.is_empty() {
            Vec::new()
        } else {
            content
                .replace("\r\n", "\n")
                .split('\n')
                .map(|s| s.to_string())
                .collect()
        };

        self.lines = lines;
        self.filename = Some(path.to_string());
        self.dirty = false;
        self.undo_snapshot = None;
        Ok(())
    }

    /// バッファ内容を指定パス（省略時は現在のファイル名）に保存する。
    pub fn save(&mut self, path: Option<&str>) -> CirResult<String> {
        let target = match path {
            Some(p) => p.to_string(),
            None => self.filename.clone().ok_or(CirPathError::NoFileName)?,
        };

        let mut content = self.lines.join("\n");
        if !self.lines.is_empty() {
            content.push('\n');
        }

        let mut file = fs::File::create(&target).map_err(|e| CirPathError::Io {
            path: target.clone(),
            source: e,
        })?;
        file.write_all(content.as_bytes())
            .map_err(|e| CirPathError::Io {
                path: target.clone(),
                source: e,
            })?;

        self.filename = Some(target.clone());
        self.dirty = false;
        Ok(target)
    }

    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// 1-indexed の行番号が有効範囲内かを検証する。
    fn validate_line(&self, n: usize) -> CirResult<()> {
        if n == 0 || n > self.lines.len() {
            return Err(CirPathError::OutOfRange {
                requested: n,
                max: self.lines.len(),
            });
        }
        Ok(())
    }

    /// 変更操作の直前に呼び、Undo用スナップショットを保存する。
    fn snapshot(&mut self) {
        self.undo_snapshot = Some(self.lines.clone());
    }

    pub fn undo(&mut self) -> CirResult<()> {
        match self.undo_snapshot.take() {
            Some(prev) => {
                self.lines = prev;
                self.dirty = true;
                Ok(())
            }
            None => Err(CirPathError::NothingToUndo),
        }
    }

    /// 指定範囲（1-indexed, 両端含む）の行を返す。
    pub fn get_range(&self, start: usize, end: usize) -> CirResult<&[String]> {
        self.validate_line(start)?;
        self.validate_line(end)?;
        if start > end {
            return Err(CirPathError::InvalidArgument(format!(
                "開始行 {} が終了行 {} より大きいです",
                start, end
            )));
        }
        Ok(&self.lines[start - 1..end])
    }

    /// n行目の直後にテキストを挿入する（n=0なら先頭に挿入）。
    pub fn insert_after(&mut self, n: usize, text: Vec<String>) -> CirResult<()> {
        if n > self.lines.len() {
            return Err(CirPathError::OutOfRange {
                requested: n,
                max: self.lines.len(),
            });
        }
        self.snapshot();
        let insert_pos = n; // n=0なら位置0（先頭）、n=kならk番目の要素の直後(=index k)
        for (offset, line) in text.into_iter().enumerate() {
            self.lines.insert(insert_pos + offset, line);
        }
        self.dirty = true;
        Ok(())
    }

    /// n行目をtextで置き換える。
    pub fn change_line(&mut self, n: usize, text: Vec<String>) -> CirResult<()> {
        self.validate_line(n)?;
        self.snapshot();
        self.lines.splice(n - 1..n, text);
        self.dirty = true;
        Ok(())
    }

    /// start〜end（1-indexed, 両端含む）の行を削除する。
    pub fn delete_range(&mut self, start: usize, end: usize) -> CirResult<()> {
        self.validate_line(start)?;
        self.validate_line(end)?;
        if start > end {
            return Err(CirPathError::InvalidArgument(format!(
                "開始行 {} が終了行 {} より大きいです",
                start, end
            )));
        }
        self.snapshot();
        self.lines.drain(start - 1..end);
        self.dirty = true;
        Ok(())
    }

    /// 部分一致で検索し、マッチした行番号(1-indexed)一覧を返す。
    pub fn find(&self, pattern: &str) -> Vec<usize> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(pattern))
            .map(|(i, _)| i + 1)
            .collect()
    }

    /// n行目のold→newを1回だけ置換する。
    pub fn substitute_line(&mut self, n: usize, old: &str, new: &str) -> CirResult<()> {
        self.validate_line(n)?;
        let idx = n - 1;
        if !self.lines[idx].contains(old) {
            return Err(CirPathError::NotFound(old.to_string()));
        }
        self.snapshot();
        self.lines[idx] = self.lines[idx].replacen(old, new, 1);
        self.dirty = true;
        Ok(())
    }

    /// 全行に対してold→newをすべて置換する。
    pub fn substitute_all(&mut self, old: &str, new: &str) -> CirResult<usize> {
        let mut count = 0usize;
        for line in &self.lines {
            count += line.matches(old).count();
        }
        if count == 0 {
            return Err(CirPathError::NotFound(old.to_string()));
        }
        self.snapshot();
        for line in self.lines.iter_mut() {
            *line = line.replace(old, new);
        }
        self.dirty = true;
        Ok(count)
    }
}
