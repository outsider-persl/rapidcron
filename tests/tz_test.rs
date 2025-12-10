/// 测试函数（示例输出）
#[cfg(test)]
mod tz_tests {
    use rapidcron::common::time::now_beijing;

    #[test]
    fn test_beijing_time() {
        let now = now_beijing();
        println!("当前北京时间: {}", now);
        assert_eq!(now.offset().local_minus_utc(), 8 * 3600);
    }
}
