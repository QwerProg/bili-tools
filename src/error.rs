# 全局错误处理
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BiliError {
    #[error("API请求失败: {0}")]
    ApiError(String),
    
    #[error("无效的凭证: {0}")]
    InvalidCookies(String),
    
    #[error("二维码已过期，请重新获取")]
    QrExpired,
    
    #[error("等待扫描二维码...")]
    QrNotScanned,
    
    #[error("网络连接错误: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("JSON数据解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("本地文件读写错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("配置文件异常: {0}")]
    ConfigError(String),
    
    #[error("找不到对应的直播间信息")]
    RoomNotFound,
    
    #[error("操作受限：该账号需要进行人脸识别验证")]
    FaceIdentificationRequired,
}

pub type Result<T> = std::result::Result<T, BiliError>;
