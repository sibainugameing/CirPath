use crate::error::{CirPathError, CirResult};

/// ユーザーが入力する行コマンドを表す。
#[derive(Debug)]
pub enum Command {
    Open(String),
    Save(Option<String>),
    Print(Option<(usize, usize)>), // None なら全行表示
    Insert(usize),
    Append(usize),
    Change(usize),
    Delete(usize, usize),
    Find(String),
    SubstituteLine(usize, String, String),
    SubstituteAll(String, String),
    Undo,
    Count,
    Help,
    Quit(bool), // true なら強制終了(q!)
    Empty,
}

/// 1行の入力文字列を Command に変換する。
pub fn parse(input: &str) -> CirResult<Command> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Command::Empty);
    }

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let head = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim();

    match head {
        "o" | "open" => {
            if rest.is_empty() {
                return Err(CirPathError::InvalidArgument(
                    "開くファイル名を指定してください: 'o <ファイル名>'".to_string(),
                ));
            }
            Ok(Command::Open(rest.to_string()))
        }
        "w" | "save" => {
            if rest.is_empty() {
                Ok(Command::Save(None))
            } else {
                Ok(Command::Save(Some(rest.to_string())))
            }
        }
        "p" | "print" | "l" | "list" => {
            if rest.is_empty() {
                Ok(Command::Print(None))
            } else {
                let nums: Vec<&str> = rest.split_whitespace().collect();
                match nums.len() {
                    1 => {
                        let n = parse_usize(nums[0])?;
                        Ok(Command::Print(Some((n, n))))
                    }
                    2 => {
                        let s = parse_usize(nums[0])?;
                        let e = parse_usize(nums[1])?;
                        Ok(Command::Print(Some((s, e))))
                    }
                    _ => Err(CirPathError::InvalidArgument(
                        "使い方: 'p [開始行] [終了行]'".to_string(),
                    )),
                }
            }
        }
        "i" | "insert" => {
            let n = if rest.is_empty() {
                0
            } else {
                parse_usize(rest)?
            };
            Ok(Command::Insert(n))
        }
        "a" | "append" => {
            let n = parse_usize(rest)?;
            Ok(Command::Append(n))
        }
        "c" | "change" => {
            let n = parse_usize(rest)?;
            Ok(Command::Change(n))
        }
        "d" | "delete" => {
            let nums: Vec<&str> = rest.split_whitespace().collect();
            match nums.len() {
                1 => {
                    let n = parse_usize(nums[0])?;
                    Ok(Command::Delete(n, n))
                }
                2 => {
                    let s = parse_usize(nums[0])?;
                    let e = parse_usize(nums[1])?;
                    Ok(Command::Delete(s, e))
                }
                _ => Err(CirPathError::InvalidArgument(
                    "使い方: 'd <行番号>' または 'd <開始> <終了>'".to_string(),
                )),
            }
        }
        "f" | "find" => {
            if rest.is_empty() {
                return Err(CirPathError::InvalidArgument(
                    "検索文字列を指定してください: 'f <文字列>'".to_string(),
                ));
            }
            Ok(Command::Find(rest.to_string()))
        }
        "s" | "sub" => parse_substitute(rest),
        "u" | "undo" => Ok(Command::Undo),
        "n" | "count" => Ok(Command::Count),
        "h" | "help" | "?" => Ok(Command::Help),
        "q" | "quit" => Ok(Command::Quit(false)),
        "q!" => Ok(Command::Quit(true)),
        _ => Err(CirPathError::InvalidCommand(head.to_string())),
    }
}

/// 's <行番号> /old/new/[g]' または 's /old/new/g' の形式を解釈する。
fn parse_substitute(rest: &str) -> CirResult<Command> {
    let (maybe_line, delim_part) = match rest.find('/') {
        Some(idx) => {
            let prefix = rest[..idx].trim();
            (prefix, &rest[idx..])
        }
        None => {
            return Err(CirPathError::InvalidArgument(
                "使い方: 's [行番号] /検索文字列/置換文字列/[g]'".to_string(),
            ))
        }
    };

    let segments: Vec<&str> = delim_part.split('/').collect();
    // "/old/new/" -> split('/') => ["", "old", "new", ""] or with trailing g => ["", "old", "new", "g"]
    if segments.len() < 3 {
        return Err(CirPathError::InvalidArgument(
            "置換の書式が不正です。'/検索文字列/置換文字列/' の形式で指定してください".to_string(),
        ));
    }
    let old = segments[1].to_string();
    let new = segments[2].to_string();
    let global_flag = segments.get(3).map(|s| s.contains('g')).unwrap_or(false);

    if old.is_empty() {
        return Err(CirPathError::InvalidArgument(
            "検索文字列が空です".to_string(),
        ));
    }

    if maybe_line.is_empty() {
        if global_flag {
            Ok(Command::SubstituteAll(old, new))
        } else {
            Err(CirPathError::InvalidArgument(
                "行番号を省略する場合は末尾に 'g' を付けて全行対象にしてください（例: s /old/new/g）"
                    .to_string(),
            ))
        }
    } else {
        let n = parse_usize(maybe_line)?;
        Ok(Command::SubstituteLine(n, old, new))
    }
}

fn parse_usize(s: &str) -> CirResult<usize> {
    s.parse::<usize>().map_err(|_| {
        CirPathError::InvalidArgument(format!("'{}' は正しい行番号（正の整数）ではありません", s))
    })
}
