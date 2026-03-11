//! Cron 定时任务模块

use anyhow::Result;
use chrono::{DateTime, Local};
use cron::Schedule;
use std::str::FromStr;

use crate::error::Error;

/// Cron 表达式解析器
pub struct CronParser {
    schedule: Schedule,
}

impl CronParser {
    pub fn new(expr: &str) -> Result<Self, Error> {
        // 字段数量必须为 6
        let field_count = expr.split_whitespace().count();
        if field_count != 6 {
            return Err(Error::CronFieldCount(format!(
                "Cron 应包含6个字段，但收到: {} 字段 ({})",
                field_count, expr
            )));
        }

        let schedule = Schedule::from_str(expr).map_err(|e| map_cron_error(expr, e.to_string()))?;

        Ok(Self { schedule })
    }

    /// 获取在指定时间窗口内的所有触发时间
    pub fn next_triggers_in_window(
        &self,
        start: DateTime<chrono::Utc>,
        end: DateTime<chrono::Utc>,
    ) -> Vec<DateTime<chrono::Utc>> {
        let local_offset = *Local::now().offset();
        let mut current = start.with_timezone(&local_offset);
        let end_fixed = end.with_timezone(&local_offset);
        let mut triggers = Vec::new();

        while let Some(next) = self.schedule.after(&current).next() {
            if next > end_fixed {
                break;
            }

            let next_utc = next.with_timezone(&chrono::Utc);
            triggers.push(next_utc);
            current = next;
        }

        triggers
    }
}

/// Cron 错误分类映射
fn map_cron_error(expr: &str, msg: String) -> Error {
    let lower = msg.to_lowercase();

    if lower.contains("unexpected") || lower.contains("parse") || lower.contains("failed to parse")
    {
        Error::CronSyntax(format!("非法 Cron 表达式: {}\n{}", expr, msg))
    } else if lower.contains("out of range")
        || lower.contains("must be less than")
        || lower.contains("invalid")
    {
        Error::CronTimeRange(format!("Cron 字段超出范围: {}\n{}", expr, msg))
    } else {
        Error::CronInternal(format!("内部错误: {}\n{}", expr, msg))
    }
}
