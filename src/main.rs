mod buffer;
mod commands;
mod editor;
mod error;

use editor::Editor;
use std::env;
use std::io::{self, BufReader};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 引数は0個（新規バッファで起動）または1個（ファイルを開いて起動）のみ許可する。
    if args.len() > 2 {
        eprintln!(
            "使い方: {} [ファイル名]",
            args.get(0).map(|s| s.as_str()).unwrap_or("cirpath")
        );
        process::exit(1);
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut editor = Editor::new(reader);

    if let Some(path) = args.get(1) {
        editor.open_initial(path);
    }

    editor.run();
}
