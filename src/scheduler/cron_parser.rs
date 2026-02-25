//! Cron 定时任务模块（不包含错误 enum）

use anyhow::Result;
use chrono::{DateTime, FixedOffset, Local};
use cron::Schedule;
use std::str::FromStr;

use crate::error::Error;

/// Cron 表达式解析器
pub struct CronParser {
    schedule: Schedule,
}

impl CronParser {
    /// 创建新的 Cron 调度器（带完整错误分类）
    pub fn new(expr: &str) -> Result<Self, Error> {
        // 字段数量必须为 6
        let field_count = expr.split_whitespace().count();
        if field_count != 6 {
            return Err(Error::CronFieldCount(format!(
                "Cron 应包含6个字段，但收到: {} 字段 ({})",
                field_count, expr
            )));
        }

        let schedule =
            Schedule::from_str(expr).map_err(|e| map_cron_error(expr, e.to_string()))?;

        Ok(Self { schedule })
    }

    /// 获取下一次触发时间（本地时区）
    pub fn next_trigger(&self) -> Option<DateTime<FixedOffset>> {
        let local_offset = Local::now().offset().clone();
        self.schedule
            .upcoming(local_offset)
            .next()
    }

    /// 获取未来 n 次触发时间（本地时区）
    pub fn next_triggers(&self, n: usize) -> Vec<DateTime<FixedOffset>> {
        let mut current = Local::now().fixed_offset();

        (0..n)
            .filter_map(|_| {
                let next = self.schedule.after(&current).next()?;
                current = next;
                Some(current)
            })
            .collect()
    }
}

/// Cron 错误分类映射
fn map_cron_error(expr: &str, msg: String) -> Error {
    let lower = msg.to_lowercase();

    if lower.contains("unexpected")
        || lower.contains("parse")
        || lower.contains("failed to parse")
    {
        Error::CronSyntax(format!("非法 Cron 表达式: {}\n{}", expr, msg))
    } else if lower.contains("out of range")
        || lower.contains("must be less than")
        || lower.contains("invalid")
    {
        Error::CronTimeRange(format!("Cron 字段超出范围: {}\n{}", expr, msg))
    } else {
        Error::CronInternal(format!(
            "内部错误: {}\n{}",
            expr, msg
        ))
    }
}

#[cfg(test)]
mod cron_test {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn valid_cron_expr() {
        let cron = CronParser::new("0/5 * * * * *").unwrap();
        let next = cron.next_trigger().unwrap();
        assert_eq!(next.offset().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn invalid_syntax_expr() {
        let cron = CronParser::new("invalid expr");
        assert!(cron.is_err());
    }

    #[test]
    fn invalid_field_count() {
        let cron = CronParser::new("* * * * *");
        assert!(cron.is_err());
    }

    #[test]
    fn invalid_time_range() {
        let cron = CronParser::new("70 * * * * *");
        assert!(cron.is_err());
    }

    #[test]
    fn multiple_triggers_strict_increasing() {
        let cron = CronParser::new("0/10 * * * * *").unwrap();
        let times = cron.next_triggers(5);

        assert!(times.len() > 1);
        for w in times.windows(2) {
            assert!(w[1] > w[0], "触发时间应递增");
        }
    }

    #[test]
    fn hourly_trigger() {
        let cron = CronParser::new("0 0 * * * *").unwrap();
        let next = cron.next_trigger().unwrap();
        assert_eq!(next.offset().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn daily_trigger() {
        let cron = CronParser::new("0 0 9 * * *").unwrap();
        let next = cron.next_trigger().unwrap();
        assert_eq!(next.offset().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn weekly_trigger() {
        let cron = CronParser::new("0 0 12 * * 1").unwrap();
        let next = cron.next_trigger().unwrap();
        assert_eq!(next.offset().local_minus_utc(), 8 * 3600);
    }

    #[test]
    fn cross_day_trigger() {
        let cron = CronParser::new("0 59 23 * * *").unwrap();
        let times = cron.next_triggers(2);

        assert_eq!(times.len(), 2);
        assert!(times[1] > times[0]);
    }

    #[test]
    fn cross_month_trigger() {
        let cron = CronParser::new("0 0 0 1 * *").unwrap();
        let times = cron.next_triggers(2);
        assert!(times[1] > times[0]);
    }

    #[test]
    fn leap_year_trigger() {
        let cron = CronParser::new("0 0 0 29 2 *").unwrap();
        let times = cron.next_triggers(2);
        assert!(times.len() >= 1);
    }

    #[test]
    fn timezone_offset_accuracy() {
        let cron = CronParser::new("*/30 * * * * *").unwrap();
        let next = cron.next_trigger().unwrap();
        let offset = next.offset().local_minus_utc();

        assert_eq!(offset, FixedOffset::east_opt(8 * 3600).unwrap().local_minus_utc());
    }

    #[test]
    fn special_char_invalid() {
        let cron = CronParser::new("@daily");
        assert!(cron.is_err());
    }
}
