use crate::buffer::TextBuffer;
use crate::commands::{self, Command};
use crate::error::{CirPathError, CirResult};
use std::io::{self, BufRead, Write};

pub struct Editor<R: BufRead> {
    buffer: TextBuffer,
    input: R,
    running: bool,
}

impl<R: BufRead> Editor<R> {
    pub fn new(input: R) -> Self {
        Editor {
            buffer: TextBuffer::new(),
            input,
            running: true,
        }
    }

    /// 起動時に対象ファイルが指定されていれば読み込む。
    pub fn open_initial(&mut self, path: &str) {
        match self.buffer.load(path) {
            Ok(()) => println!(
                "'{}' を読み込みました（{}行）",
                path,
                self.buffer.total_lines()
            ),
            Err(e) => {
                // 新規ファイルとして開始する（存在しないファイルはエラーにせず空バッファ扱い）
                println!("注意: {}", e);
                println!("新規ファイルとして '{}' を開始します", path);
                self.buffer.filename = Some(path.to_string());
            }
        }
    }

    /// メインループ。プロンプトを表示し、1コマンドずつ実行する。
    pub fn run(&mut self) {
        print_banner();
        while self.running {
            print!("cirpath> ");
            let _ = io::stdout().flush();

            let line = match self.read_line() {
                Some(l) => l,
                None => {
                    // EOF（標準入力終了）は安全に終了扱いにする
                    println!();
                    break;
                }
            };

            match commands::parse(&line) {
                Ok(cmd) => {
                    if let Err(e) = self.execute(cmd) {
                        eprintln!("エラー: {}", e);
                    }
                }
                Err(e) => eprintln!("エラー: {}", e),
            }
        }
    }

    fn read_line(&mut self) -> Option<String> {
        let mut buf = String::new();
        match self.input.read_line(&mut buf) {
            Ok(0) => None, // EOF
            Ok(_) => Some(buf.trim_end_matches(['\n', '\r']).to_string()),
            Err(_) => None,
        }
    }

    /// '.' のみの行が来るまで複数行を読み取る（挿入・変更用）。
    fn read_block(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        loop {
            match self.read_line() {
                Some(l) if l == "." => break,
                Some(l) => lines.push(l),
                None => break, // EOFでも安全に終わらせる
            }
        }
        lines
    }

    fn execute(&mut self, cmd: Command) -> CirResult<()> {
        match cmd {
            Command::Empty => Ok(()),

            Command::Open(path) => {
                if self.buffer.dirty {
                    return Err(CirPathError::UnsavedChanges);
                }
                self.buffer.load(&path)?;
                println!(
                    "'{}' を読み込みました（{}行）",
                    path,
                    self.buffer.total_lines()
                );
                Ok(())
            }

            Command::Save(path) => {
                let saved = self.buffer.save(path.as_deref())?;
                println!(
                    "'{}' に保存しました（{}行）",
                    saved,
                    self.buffer.total_lines()
                );
                Ok(())
            }

            Command::Print(range) => {
                if self.buffer.total_lines() == 0 {
                    println!("(空のバッファです)");
                    return Ok(());
                }
                let (start, end) = range.unwrap_or((1, self.buffer.total_lines()));
                let lines = self.buffer.get_range(start, end)?;
                for (i, line) in lines.iter().enumerate() {
                    println!("{:>5} | {}", start + i, line);
                }
                Ok(())
            }

            Command::Insert(n) => {
                println!(
                    "行 {} の直後にテキストを入力してください。'.' のみの行で終了。",
                    n
                );
                let text = self.read_block();
                let count = text.len();
                self.buffer.insert_after(n, text)?;
                println!("{}行を挿入しました", count);
                Ok(())
            }

            Command::Append(n) => {
                println!(
                    "行 {} の直後にテキストを追加してください。'.' のみの行で終了。",
                    n
                );
                let text = self.read_block();
                let count = text.len();
                self.buffer.insert_after(n, text)?;
                println!("{}行を追加しました", count);
                Ok(())
            }

            Command::Change(n) => {
                println!(
                    "行 {} を置き換える新しいテキストを入力してください。'.' のみの行で終了。",
                    n
                );
                let text = self.read_block();
                self.buffer.change_line(n, text)?;
                println!("行 {} を置き換えました", n);
                Ok(())
            }

            Command::Delete(s, e) => {
                self.buffer.delete_range(s, e)?;
                if s == e {
                    println!("行 {} を削除しました", s);
                } else {
                    println!("行 {} から {} を削除しました", s, e);
                }
                Ok(())
            }

            Command::Find(pattern) => {
                let hits = self.buffer.find(&pattern);
                if hits.is_empty() {
                    println!("'{}' は見つかりませんでした", pattern);
                } else {
                    for n in &hits {
                        println!("{:>5} | {}", n, self.buffer.lines[n - 1]);
                    }
                    println!("({}件ヒット)", hits.len());
                }
                Ok(())
            }

            Command::SubstituteLine(n, old, new) => {
                self.buffer.substitute_line(n, &old, &new)?;
                println!("行 {} を置換しました", n);
                Ok(())
            }

            Command::SubstituteAll(old, new) => {
                let count = self.buffer.substitute_all(&old, &new)?;
                println!("{}箇所を置換しました", count);
                Ok(())
            }

            Command::Undo => {
                self.buffer.undo()?;
                println!("直前の操作を取り消しました");
                Ok(())
            }

            Command::Count => {
                println!("総行数: {}", self.buffer.total_lines());
                Ok(())
            }

            Command::Help => {
                print_help();
                Ok(())
            }

            Command::Quit(force) => {
                if self.buffer.dirty && !force {
                    return Err(CirPathError::UnsavedChanges);
                }
                self.running = false;
                println!("CirPath を終了します");
                Ok(())
            }
        }
    }
}

fn print_banner() {
    println!("========================================");
    println!(" CirPath - シンプル行指向テキストエディタ");
    println!(" 'h' でコマンド一覧を表示します");
    println!("========================================");
}

fn print_help() {
    println!("--- CirPath コマンド一覧 ---");
    println!("o, open <file>          ファイルを開く");
    println!("w, save [file]          保存（省略時は現在のファイル）");
    println!("p, print [start] [end]  行を表示（範囲省略で全行）");
    println!("l, list                 全行表示（pと同じ）");
    println!("i, insert <n>           n行目の直後に挿入（'.'単独行で終了）");
    println!("a, append <n>           n行目の直後に追加（'.'単独行で終了）");
    println!("c, change <n>           n行目を置き換え（'.'単独行で終了）");
    println!("d, delete <n> [end]     行を削除");
    println!("f, find <text>          文字列を検索");
    println!("s <n> /old/new/         n行目内をold→newに置換（1箇所）");
    println!("s /old/new/g            全行のold→newをすべて置換");
    println!("u, undo                 直前の変更操作を取り消す");
    println!("n, count                総行数を表示");
    println!("h, help, ?              このヘルプを表示");
    println!("q, quit                 終了（未保存の変更があれば警告）");
    println!("q!                      未保存でも強制終了");
}
