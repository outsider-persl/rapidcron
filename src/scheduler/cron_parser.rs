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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_parser_new_valid_expressions() {
        let valid_expressions = vec![
            "0/5 * * * * *",
            "0 * * * * *",
            "0 0 * * * *",
            "0 0 0 * * *",
            "0 0 0 1 * *",
            "0 0 0 * * 1",
            "* * * * * *",
            "0,30 * * * * *",
            "0-10 * * * * *",
            "0/10 * * * * *",
        ];

        for expr in valid_expressions {
            assert!(CronParser::new(expr).is_ok(), "表达式 {} 应该有效", expr);
        }
    }

    #[test]
    fn test_cron_parser_new_invalid_field_count() {
        let invalid_expressions = vec![
            "0 * * * *",
            "0 0 * * *",
            "0 0 0 * *",
            "0 0 0 1 *",
            "0 * * * * * * *",
        ];

        for expr in invalid_expressions {
            let result = CronParser::new(expr);
            assert!(result.is_err(), "表达式 {} 应该无效", expr);
            match result {
                Err(Error::CronFieldCount(_)) => {}
                _ => panic!("表达式 {} 应该返回 CronFieldCount 错误", expr),
            }
        }
    }

    #[test]
    fn test_cron_parser_new_invalid_syntax() {
        let invalid_expressions = vec![
            "a * * * * * *",
            "0 x * * * * *",
            "0 * y * * * *",
        ];

        for expr in invalid_expressions {
            let result = CronParser::new(expr);
            assert!(result.is_err(), "表达式 {} 应该无效", expr);
        }
    }

    #[test]
    fn test_cron_parser_new_invalid_range() {
        let invalid_expressions = vec![
            "60 * * * * * *",
            "0 60 * * * * *",
            "0 * 24 * * * *",
            "0 * * 32 * * *",
            "0 * * * 13 * *",
            "0 * * * * 8 *",
        ];

        for expr in invalid_expressions {
            let result = CronParser::new(expr);
            assert!(result.is_err(), "表达式 {} 应该无效", expr);
        }
    }

    #[test]
    fn test_next_triggers_in_window_multiple_triggers() {
        let parser = CronParser::new("0/10 * * * * *").unwrap();
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::seconds(60);

        let triggers = parser.next_triggers_in_window(start, end);

        assert!(!triggers.is_empty(), "应该有触发时间");
        assert!(triggers.len() <= 7, "触发时间数量应该在合理范围内");
    }

    #[test]
    fn test_next_triggers_in_window_no_triggers() {
        let parser = CronParser::new("0 0 0 1 1 *").unwrap();
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::seconds(60);

        let triggers = parser.next_triggers_in_window(start, end);

        assert!(triggers.is_empty(), "不应该有触发时间");
    }

    #[test]
    fn test_next_triggers_in_window_boundary() {
        let parser = CronParser::new("0 * * * * *").unwrap();
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::seconds(59);

        let triggers = parser.next_triggers_in_window(start, end);

        assert!(triggers.len() <= 1, "边界情况下最多一个触发时间");
    }

    #[test]
    fn test_next_triggers_in_window_complex_expression() {
        let parser = CronParser::new("0,30 * * * * *").unwrap();
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::minutes(2);

        let triggers = parser.next_triggers_in_window(start, end);

        assert!(!triggers.is_empty(), "复杂表达式应该有触发时间");
        assert!(triggers.len() <= 5, "触发时间数量应该在合理范围内");
    }

    #[test]
    fn test_next_triggers_in_window_every_second() {
        let parser = CronParser::new("* * * * * *").unwrap();
        let start = chrono::Utc::now();
        let end = start + chrono::Duration::seconds(10);

        let triggers = parser.next_triggers_in_window(start, end);

        assert!(!triggers.is_empty(), "每秒执行应该有触发时间");
        assert!(triggers.len() <= 11, "触发时间数量应该在合理范围内");
    }

    #[test]
    fn test_map_cron_error_syntax() {
        let error = map_cron_error("test * * * * * *", "unexpected character".to_string());
        assert!(matches!(error, Error::CronSyntax(_)));
    }

    #[test]
    fn test_map_cron_error_range() {
        let error = map_cron_error("60 * * * * * *", "out of range".to_string());
        assert!(matches!(error, Error::CronTimeRange(_)));
    }

    #[test]
    fn test_map_cron_error_internal() {
        let error = map_cron_error("* * * * * *", "some other error".to_string());
        assert!(matches!(error, Error::CronInternal(_)));
    }
}
