use crate::api::area::fetch_area_list;
use crate::error::{BiliLiveError, Result};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Attribute, Print, SetAttribute},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, IsTerminal, Write};

pub fn get_area_choice() -> Result<u32> {
    let area_list = fetch_area_list()?;
    let data = area_list["data"]
        .as_array()
        .ok_or_else(|| BiliLiveError::Parse("无法解析分区列表".to_string()))?;

    let areas: Vec<(&str, Vec<(&str, &str)>)> = data
        .iter()
        .map(|a| {
            let name = a["name"].as_str().unwrap_or("");
            let subs: Vec<(&str, &str)> = a["list"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|s| {
                            (
                                s["name"].as_str().unwrap_or(""),
                                s["id"].as_str().unwrap_or(""),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            (name, subs)
        })
        .collect();

    if !std::io::stdin().is_terminal() {
        return Err(BiliLiveError::Input(
            "交互式分区选择需要在真实终端中运行，不支持管道输入".to_string(),
        ));
    }

    terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let result = run_selector(&areas);

    let _ = execute!(stdout(), LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    result
}

fn run_selector(areas: &[(&str, Vec<(&str, &str)>)]) -> Result<u32> {
    let mut stdout = stdout();
    let mut selected: usize = 0;
    let mut in_sub: bool = false;
    let mut parent_idx: usize = 0;
    let mut sub_selected: usize = 0;
    let mut scroll_offset: usize = 0;

    loop {
        let (term_cols, term_rows) = terminal::size()?;
        let viewport_rows = (term_rows as usize).saturating_sub(4);

        let (title, items, ids, num_cols, total_rows, col_width) = if !in_sub {
            let display_items: Vec<String> = areas.iter().map(|(n, _)| n.to_string()).collect();
            let item_ids: Vec<String> = Vec::new();
            let max_w = display_items.iter().map(|s| s.chars().count()).max().unwrap_or(10);
            let col_w = max_w + 4;
            let ncols = ((term_cols as usize).saturating_sub(1) / col_w).max(1);
            let rows = display_items.len().div_ceil(ncols);
            (
                "一级分区列表:",
                display_items,
                item_ids,
                ncols,
                rows,
                col_w,
            )
        } else {
            let (parent_name, subs) = &areas[parent_idx];
            if subs.is_empty() {
                return Err(BiliLiveError::Input(format!(
                    "分区 '{}' 没有子分区",
                    parent_name
                )));
            }
            let display_items: Vec<String> = subs
                .iter()
                .map(|(n, id)| format!("{} - {}", n, id))
                .collect();
            let item_ids: Vec<String> = subs.iter().map(|(_, id)| id.to_string()).collect();
            let max_w = display_items.iter().map(|s| s.chars().count()).max().unwrap_or(10);
            let col_w = max_w + 4;
            let ncols = ((term_cols as usize).saturating_sub(1) / col_w).max(1);
            let rows = display_items.len().div_ceil(ncols);
            (
                &format!("二级分区 ({}):", parent_name)[..],
                display_items,
                item_ids,
                ncols,
                rows,
                col_w,
            )
        };

        let cur = if !in_sub { selected } else { sub_selected };
        let cur_row = cur / num_cols;

        if viewport_rows > 0 {
            let center = cur_row.saturating_sub(viewport_rows / 2);
            scroll_offset = center.min(total_rows.saturating_sub(viewport_rows));
        }

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::FromCursorDown))?;
        writeln!(stdout, "\r{}\r", title)?;

        let end_row = (scroll_offset + viewport_rows).min(total_rows);
        for grid_row in scroll_offset..end_row {
            for col in 0..num_cols {
                let idx = grid_row * num_cols + col;
                if idx >= items.len() {
                    break;
                }
                let x = col * col_width;
                let y = 1 + (grid_row - scroll_offset);
                execute!(stdout, MoveTo(x as u16, y as u16))?;
                if idx == cur {
                    execute!(
                        stdout,
                        SetAttribute(Attribute::Reverse),
                        Print(format!(" > {}", items[idx])),
                        SetAttribute(Attribute::Reset)
                    )?;
                } else {
                    execute!(stdout, Print(format!("   {}", items[idx])))?;
                }
            }
        }

        let footer_y = 1 + (end_row - scroll_offset) + 1;
        execute!(stdout, MoveTo(0, footer_y as u16), Clear(ClearType::FromCursorDown))?;
        if in_sub {
            write!(
                stdout,
                "\r j/k/↑/↓/h/l/←/→=移动  Enter=选择  Backspace=返回  q=退出\r"
            )?;
        } else {
            write!(
                stdout,
                "\r j/k/↑/↓/h/l/←/→=移动  Enter=选择  q=退出\r"
            )?;
        }
        stdout.flush()?;

        let total = items.len();
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    let next = cur + num_cols;
                    if next < total {
                        set_sel(&mut selected, &mut sub_selected, in_sub, next);
                    } else if cur + 1 < total {
                        set_sel(&mut selected, &mut sub_selected, in_sub, cur + 1);
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let prev = cur.saturating_sub(num_cols);
                    if cur > 0 {
                        set_sel(&mut selected, &mut sub_selected, in_sub, prev);
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if cur + 1 < total {
                        set_sel(&mut selected, &mut sub_selected, in_sub, cur + 1);
                    }
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    if cur > 0 && cur % num_cols != 0 {
                        set_sel(&mut selected, &mut sub_selected, in_sub, cur - 1);
                    } else if in_sub {
                        in_sub = false;
                        scroll_offset = 0;
                    }
                }
                KeyCode::Enter => {
                    if !in_sub {
                        parent_idx = cur;
                        in_sub = true;
                        sub_selected = 0;
                        scroll_offset = 0;
                    } else {
                        let id = &ids[cur];
                        let numeric_id: String =
                            id.chars().filter(|c| c.is_numeric()).collect();
                        return numeric_id.parse::<u32>().map_err(|e| {
                            BiliLiveError::Parse(format!("分区ID转换失败: {}", e))
                        });
                    }
                }
                KeyCode::Backspace => {
                    if in_sub {
                        in_sub = false;
                        scroll_offset = 0;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    return Err(BiliLiveError::Input("用户取消选择".to_string()));
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Err(BiliLiveError::Input("用户取消选择".to_string()));
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn set_sel(selected: &mut usize, sub_selected: &mut usize, in_sub: bool, val: usize) {
    if in_sub {
        *sub_selected = val;
    } else {
        *selected = val;
    }
}
