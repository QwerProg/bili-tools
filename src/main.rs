mod api;
mod auth;
mod error;
mod live;
mod logger;
mod ui;
mod utils;

use crate::logger::init_logger;
use clap::Parser;
use error::{BiliLiveError, Result};

#[derive(Parser)]
#[command(
    name = "bt",
    author,
    version,
    about = "B站直播开播工具 — 命令行一键开播/下播",
    long_about = None
)]
struct Args {
    /// 开播（默认行为）
    #[arg(short = 's', long)]
    start: bool,

    /// 下播
    #[arg(short = 'c', long)]
    close: bool,

    /// 查看直播状态
    #[arg(long)]
    status: bool,

    /// 重新登录（清除 Cookie）
    #[arg(long)]
    restart: bool,

    /// 指定直播分区 ID，跳过交互式选择
    #[arg(long)]
    area: Option<u32>,

    /// 指定直播标题，跳过输入
    #[arg(long)]
    title: Option<String>,

    /// 显示完整推流码（不打码）
    #[arg(long)]
    show: bool,
}

fn main() {
    let args = Args::parse();
    init_logger();

    if let Err(e) = run(args) {
        user_error!("{}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    // --restart: delete cookies and re-login
    if args.restart {
        auth::cookies::delete_cookies()?;
        user_success!("已清除登录信息，重新登录");
        auth::start_login()?;
        user_success!("登录成功！");
        return Ok(());
    }

    // All other commands need login
    ensure_login()?;

    // --close: stop live
    if args.close {
        let cookies = auth::cookies::read_cookies()?;
        if !api::live::check_live_status(cookies.room_id)? {
            user_info!("当前未在直播中");
            return Ok(());
        }
        user_info!("准备关闭直播...");
        live::stop_live()?;
        return Ok(());
    }

    // --status: show live status
    if args.status {
        println!("{}", include_str!("logo.txt"));
        let cookies = auth::cookies::read_cookies()?;
        let is_live = api::live::check_live_status(cookies.room_id)?;
        if is_live {
            user_info!("当前正在直播中");
            if let Some(live_key) = cookies.live_key {
                live::stats::get_live_info(live_key)?;
            }
        } else {
            user_info!("当前未在直播中");
        }
        return Ok(());
    }

    // Default: start live
    let cookies = auth::cookies::read_cookies()?;

    // If already live, ask to stop first
    if api::live::check_live_status(cookies.room_id)? {
        user_info!("检测到当前正在直播中");
        user_prompt!("是否关闭直播？(y/n): ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| BiliLiveError::Input(format!("读取输入失败: {}", e)))?;

        if input.trim().to_lowercase() == "y" {
            user_info!("准备关闭直播...");
            live::stop_live()?;
        } else {
            user_info!("直播继续运行中，程序退出");
        }
        return Ok(());
    }

    // Area selection
    let area_id = if let Some(id) = args.area {
        id
    } else {
        user_prompt!("是否使用上次直播的分区？(回车/y=使用默认，n=选择新分区)");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| BiliLiveError::Input(format!("读取输入失败: {}", e)))?;

        if input.trim().is_empty() || input.trim().to_lowercase() == "y" {
            let (id, name) = api::live::get_recent_live()?;
            user_success!("使用上次的分区: {} - {}", name, id);
            id.parse()
                .map_err(|e| BiliLiveError::Parse(format!("分区ID转换失败: {}", e)))?
        } else {
            user_info!("选择合适的直播分区！");
            ui::get_area_choice()?
        }
    };

    // Title
    if let Some(ref t) = args.title {
        if !t.is_empty() {
            api::live::update_title(&cookies, t)?;
            user_success!("标题已更新为: {}", t);
        }
    } else {
        user_prompt!("请输入新标题（直接回车保留原标题）: ");
        let mut title = String::new();
        std::io::stdin()
            .read_line(&mut title)
            .map_err(|e| BiliLiveError::Input(format!("读取输入失败: {}", e)))?;
        let title = title.trim().to_string();
        if !title.is_empty() {
            api::live::update_title(&cookies, &title)?;
            user_success!("标题已更新为: {}", title);
        }
    }

    user_info!("开始直播！");
    live::start_live(&area_id.to_string(), args.show)?;
    Ok(())
}

fn ensure_login() -> Result<()> {
    if auth::check_status()? {
        user_success!("登录状态正常");
        return Ok(());
    }
    user_info!("需要登录，开始登录流程...");
    auth::start_login()?;
    user_success!("登录成功！");
    Ok(())
}
