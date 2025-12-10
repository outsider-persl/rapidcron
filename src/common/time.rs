// 文件路径: rapidcron/src/common/time.rs

use chrono::{DateTime, TimeZone, Utc, FixedOffset};

/// 全局固定时区：北京时区（UTC+8）
pub static BEIJING_TZ: FixedOffset = FixedOffset::east_opt(8 * 3600).unwrap();

/// 获取当前北京时间
pub fn now_beijing() -> DateTime<FixedOffset> {
    let utc_now = Utc::now();
    utc_now.with_timezone(&BEIJING_TZ)
}

/// 将任意时间转换为北京时间
pub fn to_beijing_time<T: TimeZone>(dt: DateTime<T>) -> DateTime<FixedOffset> {
    dt.with_timezone(&BEIJING_TZ)
}

/// 获取当前时间的字符串表示（格式化为 `%Y-%m-%d %H:%M:%S`）
pub fn now_str() -> String {
    now_beijing().format("%Y-%m-%d %H:%M:%S").to_string()
}


