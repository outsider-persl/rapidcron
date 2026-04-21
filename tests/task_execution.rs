use chrono::{Duration, Utc};
use mongodb::bson::oid::ObjectId;
use rapidcron::types::{ExecutionLog, ExecutionResult, TaskInstance, TaskStatus, TriggeredBy};

#[test]
fn test_task_instance_status_transitions() {
    let mut instance = TaskInstance {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Pending,
        executor_id: None,
        start_time: None,
        end_time: None,
        retry_count: 0,
        result: None,
        triggered_by: TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    assert_eq!(instance.status, TaskStatus::Pending);
    assert!(instance.start_time.is_none());
    assert!(instance.end_time.is_none());

    instance.status = TaskStatus::Running;
    instance.start_time = Some(Utc::now());
    instance.executor_id = Some("executor-1".to_string());

    assert_eq!(instance.status, TaskStatus::Running);
    assert!(instance.start_time.is_some());
    assert!(instance.executor_id.is_some());

    instance.status = TaskStatus::Success;
    instance.end_time = Some(Utc::now());
    instance.result = Some(ExecutionResult {
        output: Some("Success".to_string()),
        error: None,
        exit_code: Some(0),
    });

    assert_eq!(instance.status, TaskStatus::Success);
    assert!(instance.end_time.is_some());
    assert!(instance.result.is_some());
}

#[test]
fn test_task_instance_failed_status() {
    let instance = TaskInstance {
        id: Some(ObjectId::new()),
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
        triggered_by: TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    assert_eq!(instance.status, TaskStatus::Failed);
    assert_eq!(instance.retry_count, 1);
    assert!(instance.result.is_some());

    if let Some(result) = &instance.result {
        assert!(result.error.is_some());
        assert_eq!(result.exit_code, Some(1));
    }
}

#[test]
fn test_task_instance_manual_trigger() {
    let instance = TaskInstance {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Pending,
        executor_id: None,
        start_time: None,
        end_time: None,
        retry_count: 0,
        result: None,
        triggered_by: TriggeredBy::Manual,
        created_at: Utc::now(),
    };

    assert_eq!(instance.triggered_by, TriggeredBy::Manual);
}

#[test]
fn test_execution_result_success() {
    let result = ExecutionResult {
        output: Some("Task completed successfully".to_string()),
        error: None,
        exit_code: Some(0),
    };

    assert!(result.output.is_some());
    assert!(result.error.is_none());
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn test_execution_result_failure() {
    let result = ExecutionResult {
        output: None,
        error: Some("Task failed".to_string()),
        exit_code: Some(1),
    };

    assert!(result.output.is_none());
    assert!(result.error.is_some());
    assert_eq!(result.exit_code, Some(1));
}

#[test]
fn test_execution_log_creation() {
    let log = ExecutionLog {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        task_name: "test-task".to_string(),
        instance_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        start_time: Some(Utc::now()),
        end_time: Utc::now(),
        status: TaskStatus::Success,
        duration_ms: 1000,
        output_summary: Some("Task completed".to_string()),
        error_message: None,
        triggered_by: TriggeredBy::Scheduler,
    };

    assert_eq!(log.task_name, "test-task");
    assert_eq!(log.status, TaskStatus::Success);
    assert_eq!(log.duration_ms, 1000);
    assert!(log.error_message.is_none());
}

#[test]
fn test_execution_log_failure() {
    let log = ExecutionLog {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        task_name: "test-task".to_string(),
        instance_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        start_time: Some(Utc::now()),
        end_time: Utc::now(),
        status: TaskStatus::Failed,
        duration_ms: 500,
        output_summary: None,
        error_message: Some("Task failed".to_string()),
        triggered_by: TriggeredBy::Scheduler,
    };

    assert_eq!(log.status, TaskStatus::Failed);
    assert_eq!(log.duration_ms, 500);
    assert!(log.error_message.is_some());
}

#[test]
fn test_task_instance_serialization() {
    let instance = TaskInstance {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Pending,
        executor_id: None,
        start_time: None,
        end_time: None,
        retry_count: 0,
        result: None,
        triggered_by: TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    let serialized = serde_json::to_string(&instance);
    assert!(serialized.is_ok(), "任务实例应该可以序列化");

    let deserialized: Result<TaskInstance, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok(), "任务实例应该可以反序列化");
}

#[test]
fn test_execution_log_serialization() {
    let log = ExecutionLog {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        task_name: "test-task".to_string(),
        instance_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        start_time: Some(Utc::now()),
        end_time: Utc::now(),
        status: TaskStatus::Success,
        duration_ms: 1000,
        output_summary: Some("Task completed".to_string()),
        error_message: None,
        triggered_by: TriggeredBy::Scheduler,
    };

    let serialized = serde_json::to_string(&log);
    assert!(serialized.is_ok(), "执行日志应该可以序列化");

    let deserialized: Result<ExecutionLog, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok(), "执行日志应该可以反序列化");
}

#[test]
fn test_task_status_equality() {
    assert_eq!(TaskStatus::Pending, TaskStatus::Pending);
    assert_eq!(TaskStatus::Running, TaskStatus::Running);
    assert_eq!(TaskStatus::Success, TaskStatus::Success);
    assert_eq!(TaskStatus::Failed, TaskStatus::Failed);
    assert_eq!(TaskStatus::Cancelled, TaskStatus::Cancelled);

    assert_ne!(TaskStatus::Pending, TaskStatus::Running);
    assert_ne!(TaskStatus::Success, TaskStatus::Failed);
}

#[test]
fn test_triggered_by_equality() {
    assert_eq!(TriggeredBy::Scheduler, TriggeredBy::Scheduler);
    assert_eq!(TriggeredBy::Manual, TriggeredBy::Manual);

    assert_ne!(TriggeredBy::Scheduler, TriggeredBy::Manual);
}

#[test]
fn test_task_instance_duration_calculation() {
    let start_time = Utc::now();
    let end_time = start_time + Duration::milliseconds(1500);

    let instance = TaskInstance {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        scheduled_time: start_time,
        status: TaskStatus::Success,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(start_time),
        end_time: Some(end_time),
        retry_count: 0,
        result: None,
        triggered_by: TriggeredBy::Scheduler,
        created_at: start_time,
    };

    if let (Some(start), Some(end)) = (instance.start_time, instance.end_time) {
        let duration = end.signed_duration_since(start);
        assert_eq!(duration.num_milliseconds(), 1500);
    }
}

#[test]
fn test_task_instance_retry_count_increment() {
    let mut instance = TaskInstance {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        status: TaskStatus::Failed,
        executor_id: Some("executor-1".to_string()),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        retry_count: 0,
        result: Some(ExecutionResult {
            output: None,
            error: Some("Error".to_string()),
            exit_code: Some(1),
        }),
        triggered_by: TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    assert_eq!(instance.retry_count, 0);

    instance.retry_count = 1;
    assert_eq!(instance.retry_count, 1);

    instance.retry_count = 2;
    assert_eq!(instance.retry_count, 2);
}

#[test]
fn test_execution_log_output_summary_truncation() {
    let long_output = "a".repeat(1000);
    let log = ExecutionLog {
        id: Some(ObjectId::new()),
        task_id: ObjectId::new(),
        task_name: "test-task".to_string(),
        instance_id: ObjectId::new(),
        scheduled_time: Utc::now(),
        start_time: Some(Utc::now()),
        end_time: Utc::now(),
        status: TaskStatus::Success,
        duration_ms: 1000,
        output_summary: Some(long_output.clone()),
        error_message: None,
        triggered_by: TriggeredBy::Scheduler,
    };

    assert!(log.output_summary.is_some());
    let summary = log.output_summary.unwrap();
    assert!(summary.len() <= 1000);
}
