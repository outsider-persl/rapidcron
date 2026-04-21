use chrono::{Duration, TimeZone, Timelike, Utc};
use rapidcron::scheduler::cron_parser::CronParser;

#[test]
fn test_cron_parser_integration_end_to_end() {
    let expr = "0/5 * * * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::seconds(30);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 7, "触发时间数量应该在合理范围内");

    for (i, trigger) in triggers.iter().enumerate() {
        assert!(trigger >= &start, "触发时间应该在开始时间之后");
        assert!(trigger <= &end, "触发时间应该在结束时间之前");
        if i > 0 {
            let diff = trigger.timestamp() - triggers[i - 1].timestamp();
            assert!(diff >= 4 && diff <= 6, "触发间隔应该在 5 秒左右");
        }
    }
}

#[test]
fn test_cron_parser_integration_complex_schedule() {
    let expr = "0,15,30,45 * * * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::minutes(2);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 9, "触发时间数量应该在合理范围内");

    for trigger in &triggers {
        let seconds = trigger.timestamp() % 60;
        assert!(
            seconds == 0 || seconds == 15 || seconds == 30 || seconds == 45,
            "触发时间应该在 0、15、30、45 秒"
        );
    }
}

#[test]
fn test_cron_parser_integration_hourly_schedule() {
    let expr = "0 0 * * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::hours(25);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 26, "触发时间数量应该在合理范围内");

    for trigger in &triggers {
        assert!(trigger.timestamp() % 3600 == 0, "触发时间应该在整点");
    }
}

#[test]
fn test_cron_parser_integration_daily_schedule() {
    let expr = "0 0 0 * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::days(3);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 4, "触发时间数量应该在合理范围内");

    for trigger in &triggers {
        let minute = trigger.minute();
        let second = trigger.second();
        assert_eq!(minute, 0, "触发时间应该在零分");
        assert_eq!(second, 0, "触发时间应该在零秒");
    }
}

#[test]
fn test_cron_parser_integration_weekly_schedule() {
    let expr = "0 0 0 * * 1";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::weeks(2);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 3, "触发时间数量应该在合理范围内");

    for trigger in &triggers {
        let minute = trigger.minute();
        let second = trigger.second();
        assert_eq!(minute, 0, "触发时间应该在零分");
        assert_eq!(second, 0, "触发时间应该在零秒");
    }
}

#[test]
fn test_cron_parser_integration_monthly_schedule() {
    let expr = "0 0 0 1 * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::days(60);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 3, "触发时间数量应该在合理范围内");

    for trigger in &triggers {
        let minute = trigger.minute();
        let second = trigger.second();
        assert_eq!(minute, 0, "触发时间应该在零分");
        assert_eq!(second, 0, "触发时间应该在零秒");
    }
}

#[test]
fn test_cron_parser_integration_range_expression() {
    let expr = "0-10 * * * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc.with_ymd_and_hms(2025, 3, 12, 12, 30, 0).unwrap();
    let end = start + Duration::seconds(30);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");

    for trigger in &triggers {
        let seconds = trigger.timestamp() % 60;
        assert!(seconds >= 0 && seconds <= 10, "触发时间应该在 0-10 秒之间");
    }
}

#[test]
fn test_cron_parser_integration_step_expression() {
    let expr = "0/15 * * * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::seconds(60);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");

    for trigger in &triggers {
        let seconds = trigger.timestamp() % 60;
        assert!(seconds % 15 == 0, "触发时间应该是 15 的倍数");
    }
}

#[test]
fn test_cron_parser_integration_empty_window() {
    let expr = "0 0 0 1 1 *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::seconds(60);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(triggers.is_empty(), "不应该有触发时间");
}

#[test]
fn test_cron_parser_integration_very_long_window() {
    let expr = "* * * * * *";
    let parser = CronParser::new(expr).expect("应该成功解析 Cron 表达式");

    let start = Utc::now();
    let end = start + Duration::seconds(100);

    let triggers = parser.next_triggers_in_window(start, end);

    assert!(!triggers.is_empty(), "应该有触发时间");
    assert!(triggers.len() <= 101, "触发时间数量应该在合理范围内");
}
