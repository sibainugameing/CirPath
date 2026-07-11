use std::fmt;
use std::io;

/// CirPath 全体で使用する統一エラー型。
/// すべての失敗経路をここに集約し、パニックを起こさず
/// ユーザーに分かりやすいメッセージとして表示することを目的とする。
#[derive(Debug)]
pub enum CirPathError {
    /// ファイルI/Oに関するエラー（読み込み・書き込み・権限等）
    Io { path: String, source: io::Error },
    /// 指定された行番号がバッファの範囲外
    OutOfRange { requested: usize, max: usize },
    /// コマンドの構文が不正
    InvalidCommand(String),
    /// 引数が不足している、または解釈できない
    InvalidArgument(String),
    /// 開いているファイルが未指定の状態で保存しようとした
    NoFileName,
    /// 未保存の変更があるまま終了しようとした
    UnsavedChanges,
    /// Undoできる履歴が存在しない
    NothingToUndo,
    /// 検索・置換対象の文字列が見つからなかった
    NotFound(String),
}

impl fmt::Display for CirPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CirPathError::Io { path, source } => {
                write!(f, "I/Oエラー: '{}' の処理に失敗しました ({})", path, source)
            }
            CirPathError::OutOfRange { requested, max } => {
                write!(
                    f,
                    "範囲エラー: 行番号 {} は範囲外です（現在の総行数: {}）",
                    requested, max
                )
            }
            CirPathError::InvalidCommand(cmd) => {
                write!(
                    f,
                    "不明なコマンドです: '{}'  ('h' でヘルプを表示できます)",
                    cmd
                )
            }
            CirPathError::InvalidArgument(msg) => {
                write!(f, "引数エラー: {}", msg)
            }
            CirPathError::NoFileName => {
                write!(
                    f,
                    "保存先のファイル名が指定されていません。'w <ファイル名>' を使用してください"
                )
            }
            CirPathError::UnsavedChanges => {
                write!(
                    f,
                    "未保存の変更があります。保存する場合は 'w'、破棄して終了する場合は 'q!' を使用してください"
                )
            }
            CirPathError::NothingToUndo => {
                write!(f, "これ以上元に戻す操作がありません")
            }
            CirPathError::NotFound(pattern) => {
                write!(f, "'{}' は見つかりませんでした", pattern)
            }
        }
    }
}

impl std::error::Error for CirPathError {}

pub type CirResult<T> = Result<T, CirPathError>;
