use rapidcron::executor::retry::retry_logic::{RetryConfig, RetryStrategy};

#[test]
fn test_retry_strategy_fixed() {
    let strategy = RetryStrategy::Fixed { delay_seconds: 10 };

    for _i in 0..5 {
        let delay = match strategy {
            RetryStrategy::Fixed { delay_seconds } => delay_seconds,
            _ => panic!("应该是固定延迟策略"),
        };
        assert_eq!(delay, 10, "固定延迟应该始终是 10 秒");
    }
}

#[test]
fn test_retry_strategy_exponential() {
    let strategy = RetryStrategy::Exponential {
        base_delay_seconds: 5,
        max_delay_seconds: 300,
    };

    let delays = vec![5, 10, 20, 40, 80, 160, 300, 300, 300];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Exponential {
                base_delay_seconds,
                max_delay_seconds,
            } => (base_delay_seconds * 2_i64.pow(i as u32)).min(max_delay_seconds),
            _ => panic!("应该是指数退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}

#[test]
fn test_retry_strategy_linear() {
    let strategy = RetryStrategy::Linear {
        initial_delay_seconds: 5,
        increment_seconds: 10,
    };

    let delays = vec![5, 15, 25, 35, 45];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Linear {
                initial_delay_seconds,
                increment_seconds,
            } => initial_delay_seconds + increment_seconds * i as i64,
            _ => panic!("应该是线性退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();

    assert!(matches!(config.strategy, RetryStrategy::Exponential { .. }));
}

#[test]
fn test_retry_strategy_exponential_max_delay() {
    let strategy = RetryStrategy::Exponential {
        base_delay_seconds: 5,
        max_delay_seconds: 100,
    };

    let delays = vec![5, 10, 20, 40, 80, 100, 100, 100];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Exponential {
                base_delay_seconds,
                max_delay_seconds,
            } => (base_delay_seconds * 2_i64.pow(i as u32)).min(max_delay_seconds),
            _ => panic!("应该是指数退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}

#[test]
fn test_retry_strategy_exponential_with_small_max() {
    let strategy = RetryStrategy::Exponential {
        base_delay_seconds: 10,
        max_delay_seconds: 50,
    };

    let delays = vec![10, 20, 40, 50, 50, 50];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Exponential {
                base_delay_seconds,
                max_delay_seconds,
            } => (base_delay_seconds * 2_i64.pow(i as u32)).min(max_delay_seconds),
            _ => panic!("应该是指数退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}

#[test]
fn test_retry_strategy_linear_with_large_increment() {
    let strategy = RetryStrategy::Linear {
        initial_delay_seconds: 10,
        increment_seconds: 30,
    };

    let delays = vec![10, 40, 70, 100, 130];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Linear {
                initial_delay_seconds,
                increment_seconds,
            } => initial_delay_seconds + increment_seconds * i as i64,
            _ => panic!("应该是线性退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}

#[test]
fn test_retry_strategy_fixed_with_large_delay() {
    let strategy = RetryStrategy::Fixed { delay_seconds: 300 };

    for _i in 0..10 {
        let delay = match strategy {
            RetryStrategy::Fixed { delay_seconds } => delay_seconds,
            _ => panic!("应该是固定延迟策略"),
        };
        assert_eq!(delay, 300, "固定延迟应该始终是 300 秒");
    }
}

#[test]
fn test_retry_strategy_exponential_base_delay_zero() {
    let strategy = RetryStrategy::Exponential {
        base_delay_seconds: 0,
        max_delay_seconds: 300,
    };

    let delays = vec![0, 0, 0, 0, 0];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Exponential {
                base_delay_seconds,
                max_delay_seconds,
            } => (base_delay_seconds * 2_i64.pow(i as u32)).min(max_delay_seconds),
            _ => panic!("应该是指数退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}

#[test]
fn test_retry_strategy_linear_initial_delay_zero() {
    let strategy = RetryStrategy::Linear {
        initial_delay_seconds: 0,
        increment_seconds: 10,
    };

    let delays = vec![0, 10, 20, 30, 40];

    for (i, expected_delay) in delays.iter().enumerate() {
        let delay = match strategy {
            RetryStrategy::Linear {
                initial_delay_seconds,
                increment_seconds,
            } => initial_delay_seconds + increment_seconds * i as i64,
            _ => panic!("应该是线性退避策略"),
        };
        assert_eq!(
            delay, *expected_delay,
            "第 {} 次重试延迟应该是 {} 秒",
            i, expected_delay
        );
    }
}
