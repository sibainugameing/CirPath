/// CirPath の表示言語。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Ja,
    En,
}

impl Lang {
    pub fn from_str(s: &str) -> Lang {
        match s {
            "en" => Lang::En,
            _ => Lang::Ja,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Lang::Ja => "ja",
            Lang::En => "en",
        }
    }

    pub fn toggle(self) -> Lang {
        match self {
            Lang::Ja => Lang::En,
            Lang::En => Lang::Ja,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::Ja => "日本語",
            Lang::En => "English",
        }
    }
}

// ---- 画面フォーカス名 / Focus window labels ----
pub fn focus_editor(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "エディタ",
        Lang::En => "Editor",
    }
}
pub fn focus_browser(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "ファイルブラウザ",
        Lang::En => "File Browser",
    }
}
pub fn focus_menu(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "メニュー",
        Lang::En => "Menu",
    }
}

pub fn title_bar(l: Lang, focus_label: &str) -> String {
    match l {
        Lang::Ja => format!(
            " CirPath - {}   (Ctrl+E:次へ Ctrl+Q:前へ Ctrl+G:ヘルプ)",
            focus_label
        ),
        Lang::En => format!(
            " CirPath - {}   (Ctrl+E: next  Ctrl+Q: prev  Ctrl+G: help)",
            focus_label
        ),
    }
}

// ---- エディタ / Editor ----
pub fn ed_new_buffer(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "新規バッファ",
        Lang::En => "New buffer",
    }
}
pub fn ed_dirty_mark(l: Lang) -> &'static str {
    match l {
        Lang::Ja => " [変更あり]",
        Lang::En => " [modified]",
    }
}
pub fn ed_status_line(l: Lang, name: &str, dirty_mark: &str, row: usize, total: usize) -> String {
    match l {
        Lang::Ja => format!("{}{}  行 {}/{}", name, dirty_mark, row, total),
        Lang::En => format!("{}{}  line {}/{}", name, dirty_mark, row, total),
    }
}
pub fn ed_loaded(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "ファイルを読み込みました",
        Lang::En => "File loaded",
    }
}
pub fn ed_new_file_read_error(l: Lang, e: &str) -> String {
    match l {
        Lang::Ja => format!("新規ファイル(読込エラー: {})", e),
        Lang::En => format!("New file (read error: {})", e),
    }
}
pub fn ed_saved(l: Lang, path: &str) -> String {
    match l {
        Lang::Ja => format!("保存しました: {}", path),
        Lang::En => format!("Saved: {}", path),
    }
}
pub fn ed_save_failed(l: Lang, e: &str) -> String {
    match l {
        Lang::Ja => format!("保存に失敗しました: {}", e),
        Lang::En => format!("Failed to save: {}", e),
    }
}
pub fn ed_confirm_quit(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "未保存の変更があります。もう一度 Ctrl+X で終了",
        Lang::En => "Unsaved changes. Press Ctrl+X again to quit",
    }
}
pub fn ed_cancelled(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "キャンセルしました",
        Lang::En => "Cancelled",
    }
}
pub fn ed_search_empty(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "検索文字列が空です",
        Lang::En => "Search text is empty",
    }
}
pub fn ed_search_found(l: Lang, needle: &str) -> String {
    match l {
        Lang::Ja => format!("検索: \"{}\" が見つかりました", needle),
        Lang::En => format!("Found: \"{}\"", needle),
    }
}
pub fn ed_search_not_found(l: Lang, needle: &str) -> String {
    match l {
        Lang::Ja => format!("見つかりません: \"{}\"", needle),
        Lang::En => format!("Not found: \"{}\"", needle),
    }
}
pub fn ed_replaced(l: Lang, from: &str, to: &str, count: usize) -> String {
    match l {
        Lang::Ja => format!("\"{}\" を \"{}\" に {} 件置換しました", from, to, count),
        Lang::En => format!("Replaced {} occurrence(s) of \"{}\" with \"{}\"", count, from, to),
    }
}
pub fn ed_goto_ok(l: Lang, n: usize) -> String {
    match l {
        Lang::Ja => format!("{} 行目へ移動しました", n),
        Lang::En => format!("Moved to line {}", n),
    }
}
pub fn ed_goto_invalid(l: Lang, max: usize) -> String {
    match l {
        Lang::Ja => format!("無効な行番号です(1〜{}の範囲で指定してください)", max),
        Lang::En => format!("Invalid line number (must be between 1 and {})", max),
    }
}
pub fn ed_filename_empty(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "ファイル名が空です",
        Lang::En => "Filename is empty",
    }
}
pub fn ed_inserted(l: Lang, path: &str) -> String {
    match l {
        Lang::Ja => format!("挿入しました: {}", path),
        Lang::En => format!("Inserted: {}", path),
    }
}
pub fn ed_insert_failed(l: Lang, e: &str) -> String {
    match l {
        Lang::Ja => format!("読み込みに失敗しました: {}", e),
        Lang::En => format!("Failed to read: {}", e),
    }
}
pub fn ed_no_clipboard(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "貼り付けるデータがありません",
        Lang::En => "Nothing to paste",
    }
}
pub fn ed_pasted(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "貼り付けました",
        Lang::En => "Pasted",
    }
}
pub fn ed_cursor_pos(l: Lang, row: usize, col: usize, total: usize) -> String {
    match l {
        Lang::Ja => format!("カーソル位置: {} 行 {} 列 (全 {} 行)", row, col, total),
        Lang::En => format!("Cursor: line {} col {} (of {} lines)", row, col, total),
    }
}
pub fn ed_prompt_search(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "検索文字列",
        Lang::En => "Search",
    }
}
pub fn ed_prompt_replace_from(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "置換: 検索文字列",
        Lang::En => "Replace: search for",
    }
}
pub fn ed_prompt_replace_to(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "置換: 置換後の文字列",
        Lang::En => "Replace: replace with",
    }
}
pub fn ed_prompt_goto(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "移動先の行番号",
        Lang::En => "Go to line",
    }
}
pub fn ed_prompt_saveas(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "保存先ファイル名",
        Lang::En => "Save as",
    }
}
pub fn ed_prompt_insertfile(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "挿入するファイルのパス",
        Lang::En => "File to insert",
    }
}

/// nano/pico 風の下部ショートカット一覧。(キー表示, ラベル) のペアを列優先で12個返す。
pub fn ed_shortcut_grid(l: Lang) -> Vec<(&'static str, &'static str)> {
    match l {
        Lang::Ja => vec![
            ("^G", "ヘルプ"),
            ("^X", "終了"),
            ("^S", "保存"),
            ("^O", "別名保存"),
            ("^W", "検索"),
            ("^\\", "置換"),
            ("^R", "ファイル挿入"),
            ("^_", "行移動"),
            ("^K", "切り取り"),
            ("^U", "貼り付け"),
            ("^C", "位置表示"),
            ("", ""),
        ],
        Lang::En => vec![
            ("^G", "Get Help"),
            ("^X", "Exit"),
            ("^S", "Save"),
            ("^O", "Save As"),
            ("^W", "Where Is"),
            ("^\\", "Replace"),
            ("^R", "Read File"),
            ("^_", "Go To Line"),
            ("^K", "Cut Text"),
            ("^U", "UnCut Text"),
            ("^C", "Cur Pos"),
            ("", ""),
        ],
    }
}

// ---- ファイルブラウザ / File browser ----
pub fn br_hint(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "up/down:選択 enter/right:開く backspace/left/u:上へ g:パス移動 n:新規ファイル N:新規フォルダ r:名前変更 d:削除",
        Lang::En => "up/down:select enter/right:open backspace/left/u:up g:goto path n:new file N:new folder r:rename d:delete",
    }
}
pub fn br_moved(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "移動しました",
        Lang::En => "Moved",
    }
}
pub fn br_path_not_found(l: Lang, path: &str) -> String {
    match l {
        Lang::Ja => format!("パスが見つかりません: {}", path),
        Lang::En => format!("Path not found: {}", path),
    }
}
pub fn br_input_empty(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "入力が空です",
        Lang::En => "Input is empty",
    }
}
pub fn br_file_exists(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "同名のファイルが既に存在します",
        Lang::En => "A file with that name already exists",
    }
}
pub fn br_folder_exists(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "同名のフォルダが既に存在します",
        Lang::En => "A folder with that name already exists",
    }
}
pub fn br_created_file(l: Lang, name: &str) -> String {
    match l {
        Lang::Ja => format!("作成しました: {}", name),
        Lang::En => format!("Created: {}", name),
    }
}
pub fn br_created_folder(l: Lang, name: &str) -> String {
    match l {
        Lang::Ja => format!("フォルダを作成しました: {}", name),
        Lang::En => format!("Folder created: {}", name),
    }
}
pub fn br_create_failed(l: Lang, e: &str) -> String {
    match l {
        Lang::Ja => format!("作成に失敗しました: {}", e),
        Lang::En => format!("Failed to create: {}", e),
    }
}
pub fn br_renamed(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "名前を変更しました",
        Lang::En => "Renamed",
    }
}
pub fn br_rename_failed(l: Lang, e: &str) -> String {
    match l {
        Lang::Ja => format!("変更に失敗しました: {}", e),
        Lang::En => format!("Failed to rename: {}", e),
    }
}
pub fn br_no_selection(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "対象が選択されていません",
        Lang::En => "Nothing is selected",
    }
}
pub fn br_deleted(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "削除しました",
        Lang::En => "Deleted",
    }
}
pub fn br_delete_failed(l: Lang, e: &str) -> String {
    match l {
        Lang::Ja => format!("削除に失敗しました: {}", e),
        Lang::En => format!("Failed to delete: {}", e),
    }
}
pub fn br_delete_cancelled(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "削除をキャンセルしました",
        Lang::En => "Delete cancelled",
    }
}
pub fn br_confirm_delete(l: Lang) -> &'static str {
    match l {
        Lang::Ja => " 本当に削除しますか? (y: はい / それ以外: キャンセル)",
        Lang::En => " Really delete this? (y: yes / anything else: cancel)",
    }
}
pub fn br_cancelled(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "キャンセルしました",
        Lang::En => "Cancelled",
    }
}
pub fn br_prompt_goto(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "移動先パス(相対/絶対)",
        Lang::En => "Path to go to (relative/absolute)",
    }
}
pub fn br_prompt_new_file(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "新規ファイル名",
        Lang::En => "New file name",
    }
}
pub fn br_prompt_new_folder(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "新規フォルダ名",
        Lang::En => "New folder name",
    }
}
pub fn br_prompt_rename(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "新しい名前",
        Lang::En => "New name",
    }
}

// ---- メニュー / Menu ----
pub fn menu_categories(l: Lang) -> Vec<&'static str> {
    match l {
        Lang::Ja => vec![
            "一般",
            "エディタ",
            "ファイルブラウザ",
            "キー操作一覧",
            "設定ファイル",
            "このアプリについて",
        ],
        Lang::En => vec![
            "General",
            "Editor",
            "File Browser",
            "Key Bindings",
            "Config File",
            "About",
        ],
    }
}

pub fn menu_hint(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "left/right:パネル切替  up/down:項目選択  Enter:決定/切替",
        Lang::En => "left/right: switch panel  up/down: select  Enter: apply/toggle",
    }
}

pub fn on_off(l: Lang, b: bool) -> &'static str {
    match (l, b) {
        (Lang::Ja, true) => "ON",
        (Lang::Ja, false) => "OFF",
        (Lang::En, true) => "ON",
        (Lang::En, false) => "OFF",
    }
}

pub fn menu_item_general_hint(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "設定はこのメニューから即座に反映されます",
        Lang::En => "Settings here take effect immediately",
    }
}
pub fn menu_item_language(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "表示言語を切り替える (日本語 / English)",
        Lang::En => "Switch display language (Japanese / English)",
    }
}
pub fn menu_item_line_numbers(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "行番号を表示する",
        Lang::En => "Show line numbers",
    }
}
pub fn menu_item_tab_width(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "タブ幅を変更する (押すたびに+1、8の次で1に戻る)",
        Lang::En => "Change tab width (increases by 1 each press, wraps after 8)",
    }
}
pub fn menu_item_trailing_newline(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "保存時に末尾改行を保証する",
        Lang::En => "Ensure trailing newline on save",
    }
}
pub fn menu_item_auto_open(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "ファイル選択時に自動でエディタへ切り替える",
        Lang::En => "Auto-switch to editor when a file is selected",
    }
}
pub fn menu_item_show_hidden(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "隠しファイル(ドットファイル)を表示する",
        Lang::En => "Show hidden (dot) files",
    }
}
pub fn menu_item_open_config(l: Lang) -> &'static str {
    match l {
        Lang::Ja => "設定ファイルをエディタで開いて直接編集する",
        Lang::En => "Open the config file in the editor to edit directly",
    }
}
pub fn menu_about(l: Lang) -> Vec<String> {
    match l {
        Lang::Ja => vec![
            "CirPath - nano風 TUI テキストエディタ".to_string(),
            "エディタ / ファイルブラウザ / メニューの3画面構成".to_string(),
            "Rust + ratatui + crossterm で実装".to_string(),
        ],
        Lang::En => vec![
            "CirPath - a nano-inspired TUI text editor".to_string(),
            "Three windows: Editor / File Browser / Menu".to_string(),
            "Built with Rust + ratatui + crossterm".to_string(),
        ],
    }
}
pub fn menu_keybinds(l: Lang) -> Vec<String> {
    match l {
        Lang::Ja => vec![
            "Ctrl+E : 次のウィンドウへ切り替え".to_string(),
            "Ctrl+Q : 前のウィンドウへ切り替え".to_string(),
            "Ctrl+G : ヘルプ(このキー操作一覧を表示)".to_string(),
            "[エディタ] Ctrl+S : 保存  Ctrl+O : 別名で保存".to_string(),
            "[エディタ] Ctrl+X : 終了 (未保存時は2回押し)".to_string(),
            "[エディタ] Ctrl+W : 検索  Ctrl+\\ : 置換".to_string(),
            "[エディタ] Ctrl+K : 行を切り取り  Ctrl+U : 貼り付け".to_string(),
            "[エディタ] Ctrl+_ : 指定行へ移動".to_string(),
            "[エディタ] Ctrl+R : 指定ファイルをカーソル位置へ挿入".to_string(),
            "[エディタ] Ctrl+C : カーソル位置(行/列)を表示".to_string(),
            "[ブラウザ] up/down : 選択移動".to_string(),
            "[ブラウザ] enter/right : 開く   backspace/left/u : 上の階層へ".to_string(),
            "[ブラウザ] g : 絶対/相対パスを直接入力して移動".to_string(),
            "[ブラウザ] n : 新規ファイル作成   N : 新規フォルダ作成".to_string(),
            "[ブラウザ] r : 名前変更   d : 削除(確認あり)".to_string(),
            "[ブラウザ] Ctrl+H : 隠しファイル表示切替".to_string(),
            "[メニュー] left/right : 左右パネル切替   up/down : 項目選択   Enter : 決定/切替".to_string(),
        ],
        Lang::En => vec![
            "Ctrl+E : switch to the next window".to_string(),
            "Ctrl+Q : switch to the previous window".to_string(),
            "Ctrl+G : help (shows this key binding list)".to_string(),
            "[Editor] Ctrl+S : save   Ctrl+O : save as".to_string(),
            "[Editor] Ctrl+X : quit (press twice if unsaved)".to_string(),
            "[Editor] Ctrl+W : search   Ctrl+\\ : replace".to_string(),
            "[Editor] Ctrl+K : cut line   Ctrl+U : paste".to_string(),
            "[Editor] Ctrl+_ : go to line".to_string(),
            "[Editor] Ctrl+R : insert a file at the cursor".to_string(),
            "[Editor] Ctrl+C : show cursor position (line/col)".to_string(),
            "[Browser] up/down : move selection".to_string(),
            "[Browser] enter/right : open   backspace/left/u : go up".to_string(),
            "[Browser] g : go to an absolute/relative path directly".to_string(),
            "[Browser] n : new file   N : new folder".to_string(),
            "[Browser] r : rename   d : delete (with confirmation)".to_string(),
            "[Browser] Ctrl+H : toggle hidden files".to_string(),
            "[Menu] left/right : switch panel   up/down : select item   Enter : apply/toggle".to_string(),
        ],
    }
}
