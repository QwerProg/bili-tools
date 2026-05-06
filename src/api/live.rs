use crate::api::client::DEFAULT_USER_AGENT;
use crate::auth::cookies::{read_cookies, Cookies};
use crate::error::{BiliLiveError, Result};

pub fn check_live_status(room_id: i32) -> Result<bool> {
    let url = format!(
        "https://api.live.bilibili.com/room/v1/Room/get_info?room_id={}",
        room_id
    );
    let response = minreq::get(&url)
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .send()?;

    let response_text = response.as_str()?;
    let json: serde_json::Value = serde_json::from_str(response_text)?;
    let live_status = json["data"]["live_status"]
        .as_i64()
        .ok_or_else(|| BiliLiveError::Parse("无法解析直播状态".to_string()))?;
    Ok(live_status == 1)
}

pub fn get_recent_live() -> Result<(String, String)> {
    let room_id = read_cookies()?.room_id;
    let url = format!(
        "https://api.live.bilibili.com/room/v1/Area/getMyChooseArea?roomid={}",
        room_id
    );
    let response = minreq::get(&url)
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .send()?;

    let response_text = response.as_str()?;
    let json: serde_json::Value = serde_json::from_str(response_text)?;
    let data = &json["data"][0];
    let id = data["id"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("无法解析分区ID".to_string()))?
        .to_string();
    let name = data["name"]
        .as_str()
        .ok_or_else(|| BiliLiveError::Parse("无法解析分区名称".to_string()))?
        .to_string();
    Ok((id, name))
}

pub fn update_title(cookies: &Cookies, title: &str) -> Result<()> {
    let form = format!(
        "room_id={}&title={}&csrf_token={}&csrf={}",
        cookies.room_id, title, cookies.csrf_token, cookies.csrf_token
    );
    let response = minreq::post("https://api.live.bilibili.com/room/v1/Room/update")
        .with_header("User-Agent", DEFAULT_USER_AGENT)
        .with_header("Content-Type", "application/x-www-form-urlencoded")
        .with_header("Cookie", format!("SESSDATA={}", cookies.sessdata))
        .with_body(form)
        .send()?;

    let json: serde_json::Value = serde_json::from_str(response.as_str()?)?;
    if json["code"].as_i64() != Some(0) {
        return Err(BiliLiveError::Api(format!(
            "更新标题失败: {}",
            json["message"].as_str().unwrap_or("未知错误")
        )));
    }
    Ok(())
}
