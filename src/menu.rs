#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    None,
    ToggleAutoJump,
    ToggleLineNumbers,
    IncreaseTabWidth,
    DecreaseTabWidth,
    OpenConfigFile,
    SetThemeDark,
    SetThemeLight,
    Quit,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
}

#[derive(Debug, Clone)]
pub struct MenuCategory {
    pub name: String,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuFocus {
    Category,
    Item,
}

pub struct Menu {
    pub categories: Vec<MenuCategory>,
    pub selected_category: usize,
    pub selected_item: usize,
    pub focus: MenuFocus,
}

impl Menu {
    pub fn new() -> Self {
        let categories = vec![
            MenuCategory {
                name: "ファイル".to_string(),
                items: vec![
                    MenuItem { label: "設定ファイルを開く (Config編集)".to_string(), action: MenuAction::OpenConfigFile },
                    MenuItem { label: "終了".to_string(), action: MenuAction::Quit },
                ],
            },
            MenuCategory {
                name: "エディタ設定".to_string(),
                items: vec![
                    MenuItem { label: "行番号表示 切替".to_string(), action: MenuAction::ToggleLineNumbers },
                    MenuItem { label: "タブ幅を増やす (+1)".to_string(), action: MenuAction::IncreaseTabWidth },
                    MenuItem { label: "タブ幅を減らす (-1)".to_string(), action: MenuAction::DecreaseTabWidth },
                ],
            },
            MenuCategory {
                name: "ブラウザ設定".to_string(),
                items: vec![
                    MenuItem { label: "選択で自動的にエディタへ移動 切替".to_string(), action: MenuAction::ToggleAutoJump },
                ],
            },
            MenuCategory {
                name: "表示".to_string(),
                items: vec![
                    MenuItem { label: "テーマ: ダーク".to_string(), action: MenuAction::SetThemeDark },
                    MenuItem { label: "テーマ: ライト".to_string(), action: MenuAction::SetThemeLight },
                ],
            },
        ];
        Menu {
            categories,
            selected_category: 0,
            selected_item: 0,
            focus: MenuFocus::Category,
        }
    }

    pub fn move_up(&mut self) {
        match self.focus {
            MenuFocus::Category => {
                if self.selected_category > 0 {
                    self.selected_category -= 1;
                    self.selected_item = 0;
                }
            }
            MenuFocus::Item => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.focus {
            MenuFocus::Category => {
                if self.selected_category + 1 < self.categories.len() {
                    self.selected_category += 1;
                    self.selected_item = 0;
                }
            }
            MenuFocus::Item => {
                let max = self.categories[self.selected_category].items.len();
                if max > 0 && self.selected_item + 1 < max {
                    self.selected_item += 1;
                }
            }
        }
    }

    pub fn focus_right(&mut self) {
        if !self.categories[self.selected_category].items.is_empty() {
            self.focus = MenuFocus::Item;
        }
    }

    pub fn focus_left(&mut self) {
        self.focus = MenuFocus::Category;
    }

    pub fn activate(&mut self) -> MenuAction {
        if self.focus == MenuFocus::Item {
            if let Some(item) = self.categories[self.selected_category]
                .items
                .get(self.selected_item)
            {
                return item.action.clone();
            }
        } else {
            self.focus_right();
        }
        MenuAction::None
    }
}
