use ratatui::style::{Color, Modifier, Style};

/// エディタ / ファイルブラウザの下部ステータス表示で使う、種類つきメッセージ。
/// エラー時は赤で表示し、それ以外(情報)は白背景・黒文字で表示する。
#[derive(Clone)]
pub struct StatusMsg {
    pub text: String,
    pub kind: MsgKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    Info,
    Error,
}

impl StatusMsg {
    pub fn info<S: Into<String>>(text: S) -> Self {
        StatusMsg {
            text: text.into(),
            kind: MsgKind::Info,
        }
    }

    pub fn error<S: Into<String>>(text: S) -> Self {
        StatusMsg {
            text: text.into(),
            kind: MsgKind::Error,
        }
    }

    /// ステータスバー用のスタイルを返す。エラーは赤背景+白文字で強調する。
    pub fn style(&self) -> Style {
        match self.kind {
            MsgKind::Info => Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
            MsgKind::Error => Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        }
    }
}
