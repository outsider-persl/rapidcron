use chrono::Utc;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use mongodb::bson::oid::ObjectId;
use rapidcron::executor::retry::retry_logic::{RetryConfig, RetryStrategy};
use rapidcron::types::{ExecutionResult, Task, TaskInstance, TaskPayload, TaskStatus, TaskType};

fn bench_retry_strategy_fixed(c: &mut Criterion) {
    let strategy = RetryStrategy::Fixed { delay_seconds: 10 };

    c.bench_function("retry_strategy_fixed", |b| {
        b.iter(|| {
            let _ = match strategy {
                RetryStrategy::Fixed { delay_seconds } => delay_seconds,
                _ => 10,
            };
        });
    });
}

fn bench_retry_strategy_exponential(c: &mut Criterion) {
    let strategy = RetryStrategy::Exponential {
        base_delay_seconds: 5,
        max_delay_seconds: 300,
    };

    c.bench_function("retry_strategy_exponential", |b| {
        b.iter(|| {
            for i in 0..10 {
                let _ = match strategy {
                    RetryStrategy::Exponential {
                        base_delay_seconds,
                        max_delay_seconds,
                    } => (base_delay_seconds * 2_i64.pow(i)).min(max_delay_seconds),
                    _ => 10,
                };
            }
        });
    });
}

fn bench_retry_strategy_linear(c: &mut Criterion) {
    let strategy = RetryStrategy::Linear {
        initial_delay_seconds: 5,
        increment_seconds: 10,
    };

    c.bench_function("retry_strategy_linear", |b| {
        b.iter(|| {
            for i in 0..10 {
                let _ = match strategy {
                    RetryStrategy::Linear {
                        initial_delay_seconds,
                        increment_seconds,
                    } => initial_delay_seconds + increment_seconds * i as i64,
                    _ => 10,
                };
            }
        });
    });
}

fn bench_should_retry_with_error(c: &mut Criterion) {
    let task = Task {
        id: None,
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'test'".to_string(),
            timeout_seconds: None,
        },
        timeout_seconds: Some(30),
        max_retries: Some(3),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let instance = TaskInstance {
        id: None,
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Failed,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        retry_count: 1,
        result: Some(ExecutionResult {
            output: None,
            error: Some("Error occurred".to_string()),
            exit_code: Some(1),
        }),
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    let _config = RetryConfig::default();

    c.bench_function("should_retry_with_error", |b| {
        b.iter(|| {
            let max_retries = task.max_retries.unwrap_or(3);
            let should_retry = max_retries > 0
                && instance.retry_count < max_retries
                && instance
                    .result
                    .as_ref()
                    .and_then(|r| r.error.as_ref())
                    .is_some();
            let _ = black_box(should_retry);
        });
    });
}

fn bench_should_retry_exceeded(c: &mut Criterion) {
    let task = Task {
        id: None,
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'test'".to_string(),
            timeout_seconds: None,
        },
        timeout_seconds: Some(30),
        max_retries: Some(2),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let instance = TaskInstance {
        id: None,
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Failed,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        retry_count: 2,
        result: Some(ExecutionResult {
            output: None,
            error: Some("Error occurred".to_string()),
            exit_code: Some(1),
        }),
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    c.bench_function("should_retry_exceeded", |b| {
        b.iter(|| {
            let max_retries = task.max_retries.unwrap_or(3);
            let should_retry = max_retries > 0
                && instance.retry_count < max_retries
                && instance
                    .result
                    .as_ref()
                    .and_then(|r| r.error.as_ref())
                    .is_some();
            let _ = black_box(should_retry);
        });
    });
}

fn bench_calculate_retry_delay_fixed(c: &mut Criterion) {
    let _task = Task {
        id: None,
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'test'".to_string(),
            timeout_seconds: None,
        },
        timeout_seconds: Some(30),
        max_retries: Some(3),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let instance = TaskInstance {
        id: None,
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Failed,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        retry_count: 2,
        result: Some(ExecutionResult {
            output: None,
            error: Some("Error occurred".to_string()),
            exit_code: Some(1),
        }),
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    let config = RetryConfig {
        strategy: RetryStrategy::Fixed { delay_seconds: 10 },
    };

    c.bench_function("calculate_retry_delay_fixed", |b| {
        b.iter(|| {
            let retry_count = instance.retry_count;
            let delay = match config.strategy {
                RetryStrategy::Fixed { delay_seconds } => delay_seconds,
                _ => 10,
            };
            let _ = black_box((retry_count, delay));
        });
    });
}

fn bench_calculate_retry_delay_exponential(c: &mut Criterion) {
    let _task = Task {
        id: None,
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'test'".to_string(),
            timeout_seconds: None,
        },
        timeout_seconds: Some(30),
        max_retries: Some(5),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let instance = TaskInstance {
        id: None,
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Failed,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        retry_count: 2,
        result: Some(ExecutionResult {
            output: None,
            error: Some("Error occurred".to_string()),
            exit_code: Some(1),
        }),
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    let config = RetryConfig {
        strategy: RetryStrategy::Exponential {
            base_delay_seconds: 5,
            max_delay_seconds: 300,
        },
    };

    c.bench_function("calculate_retry_delay_exponential", |b| {
        b.iter(|| {
            let retry_count = instance.retry_count;
            let delay = match config.strategy {
                RetryStrategy::Exponential {
                    base_delay_seconds,
                    max_delay_seconds,
                } => (base_delay_seconds * 2_i64.pow(retry_count as u32)).min(max_delay_seconds),
                _ => 10,
            };
            let _ = black_box((retry_count, delay));
        });
    });
}

fn bench_calculate_retry_delay_linear(c: &mut Criterion) {
    let _task = Task {
        id: None,
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'test'".to_string(),
            timeout_seconds: None,
        },
        timeout_seconds: Some(30),
        max_retries: Some(5),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let instance = TaskInstance {
        id: None,
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Failed,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        retry_count: 3,
        result: Some(ExecutionResult {
            output: None,
            error: Some("Error occurred".to_string()),
            exit_code: Some(1),
        }),
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    let config = RetryConfig {
        strategy: RetryStrategy::Linear {
            initial_delay_seconds: 5,
            increment_seconds: 10,
        },
    };

    c.bench_function("calculate_retry_delay_linear", |b| {
        b.iter(|| {
            let retry_count = instance.retry_count;
            let delay = match config.strategy {
                RetryStrategy::Linear {
                    initial_delay_seconds,
                    increment_seconds,
                } => initial_delay_seconds + increment_seconds * retry_count as i64,
                _ => 10,
            };
            let _ = black_box((retry_count, delay));
        });
    });
}

fn bench_batch_retry_calculation(c: &mut Criterion) {
    let instances: Vec<TaskInstance> = (0..100)
        .map(|i| TaskInstance {
            id: None,
            task_id: ObjectId::new(),
            scheduled_time: Utc::now(),
            status: TaskStatus::Failed,
            executor_id: Some(format!("executor-{}", i % 5)),
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            retry_count: i,
            result: Some(ExecutionResult {
                output: None,
                error: Some("Error occurred".to_string()),
                exit_code: Some(1),
            }),
            triggered_by: rapidcron::types::TriggeredBy::Scheduler,
            created_at: Utc::now(),
        })
        .collect();

    let config = RetryConfig {
        strategy: RetryStrategy::Exponential {
            base_delay_seconds: 5,
            max_delay_seconds: 300,
        },
    };

    c.bench_function("batch_retry_calculation", |b| {
        b.iter(|| {
            for instance in &instances {
                let retry_count = instance.retry_count;
                let delay = match config.strategy {
                    RetryStrategy::Exponential {
                        base_delay_seconds,
                        max_delay_seconds,
                    } => {
                        (base_delay_seconds * 2_i64.pow(retry_count as u32)).min(max_delay_seconds)
                    }
                    _ => 10,
                };
                let _ = black_box((retry_count, delay));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_retry_strategy_fixed,
    bench_retry_strategy_exponential,
    bench_retry_strategy_linear,
    bench_should_retry_with_error,
    bench_should_retry_exceeded,
    bench_calculate_retry_delay_fixed,
    bench_calculate_retry_delay_exponential,
    bench_calculate_retry_delay_linear,
    bench_batch_retry_calculation
);
criterion_main!(benches);
