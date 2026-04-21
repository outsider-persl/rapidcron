use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use rapidcron::types::{CreateTaskRequest, Task, TaskPayload, TaskType, UpdateTaskRequest};

#[test]
fn test_create_task_request_to_task_command() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: Some("Test task".to_string()),
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let task = request.to_task().expect("应该成功创建任务");

    assert_eq!(task.name, "test-task");
    assert_eq!(task.description, Some("Test task".to_string()));
    assert_eq!(task.task_type, TaskType::Command);
    assert_eq!(task.schedule, "0/5 * * * * *");
    assert!(task.enabled);
    assert!(matches!(task.payload, TaskPayload::Command { .. }));
}

#[test]
fn test_create_task_request_to_task_http() {
    let request = CreateTaskRequest {
        name: "http-task".to_string(),
        description: Some("HTTP task".to_string()),
        dependency_ids: vec![],
        task_type: Some("http".to_string()),
        schedule: "0 * * * * *".to_string(),
        enabled: true,
        command: None,
        url: Some("http://example.com/api".to_string()),
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let task = request.to_task().expect("应该成功创建任务");

    assert_eq!(task.name, "http-task");
    assert_eq!(task.task_type, TaskType::Http);
    assert!(matches!(task.payload, TaskPayload::Http { .. }));
}

#[test]
fn test_create_task_request_empty_name() {
    let request = CreateTaskRequest {
        name: "".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "空名称应该返回错误");
}

#[test]
fn test_create_task_request_name_too_long() {
    let request = CreateTaskRequest {
        name: "a".repeat(101),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "名称过长应该返回错误");
}

#[test]
fn test_create_task_request_invalid_cron() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "invalid-cron".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "无效的 Cron 表达式应该返回错误");
}

#[test]
fn test_create_task_request_invalid_timeout() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(0),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "超时时间小于等于0应该返回错误");
}

#[test]
fn test_create_task_request_timeout_too_large() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(3601),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "超时时间过大应该返回错误");
}

#[test]
fn test_create_task_request_invalid_max_retries() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(11),
    };

    let result = request.to_task();
    assert!(result.is_err(), "最大重试次数过大应该返回错误");
}

#[test]
fn test_create_task_request_http_without_url() {
    let request = CreateTaskRequest {
        name: "http-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("http".to_string()),
        schedule: "0 * * * * *".to_string(),
        enabled: true,
        command: None,
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "HTTP 任务没有 URL 应该返回错误");
}

#[test]
fn test_create_task_request_command_without_command() {
    let request = CreateTaskRequest {
        name: "command-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0 * * * * *".to_string(),
        enabled: true,
        command: None,
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let result = request.to_task();
    assert!(result.is_err(), "命令任务没有命令应该返回错误");
}

#[test]
fn test_create_task_request_with_dependency_ids() {
    let dep_id = ObjectId::new();
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![dep_id.to_hex()],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let task = request.to_task().expect("应该成功创建任务");

    assert_eq!(task.dependency_ids.len(), 1);
    assert_eq!(task.dependency_ids[0], dep_id);
}

#[test]
fn test_create_task_request_with_invalid_dependency_ids() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec!["invalid-id".to_string()],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let task = request.to_task().expect("应该成功创建任务");

    assert_eq!(task.dependency_ids.len(), 0);
}

#[test]
fn test_create_task_request_default_enabled() {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: None,
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: false,
        command: Some("echo 'Hello'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    let task = request.to_task().expect("应该成功创建任务");

    assert!(!task.enabled);
}

#[test]
fn test_update_task_request_partial_update() {
    let request = UpdateTaskRequest {
        name: Some("updated-task".to_string()),
        description: None,
        dependency_ids: None,
        schedule: None,
        enabled: Some(false),
        task_type: None,
        command: None,
        url: None,
        timeout_seconds: None,
        max_retries: None,
    };

    assert_eq!(request.name, Some("updated-task".to_string()));
    assert_eq!(request.enabled, Some(false));
    assert!(request.description.is_none());
}

#[test]
fn test_parse_object_id_valid() {
    let id_str = "507f1f77bcf86cd799439011";
    let result = rapidcron::types::parse_object_id(id_str);

    assert!(result.is_ok(), "有效的 ObjectId 应该解析成功");
}

#[test]
fn test_parse_object_id_invalid() {
    let id_str = "invalid-id";
    let result = rapidcron::types::parse_object_id(id_str);

    assert!(result.is_err(), "无效的 ObjectId 应该解析失败");
}

#[test]
fn test_parse_object_ids_valid() {
    let ids = vec![
        "507f1f77bcf86cd799439011".to_string(),
        "507f1f77bcf86cd799439012".to_string(),
    ];
    let result = rapidcron::types::parse_object_ids(&ids);

    assert_eq!(result.len(), 2);
}

#[test]
fn test_parse_object_ids_mixed() {
    let ids = vec![
        "507f1f77bcf86cd799439011".to_string(),
        "invalid-id".to_string(),
        "507f1f77bcf86cd799439012".to_string(),
    ];
    let result = rapidcron::types::parse_object_ids(&ids);

    assert_eq!(result.len(), 2);
}

#[test]
fn test_task_serialization() {
    let task = Task {
        id: Some(ObjectId::new()),
        name: "test-task".to_string(),
        description: Some("Test task".to_string()),
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'Hello'".to_string(),
            timeout_seconds: Some(30),
        },
        timeout_seconds: Some(30),
        max_retries: Some(3),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let serialized = serde_json::to_string(&task);
    assert!(serialized.is_ok(), "任务应该可以序列化");

    let deserialized: Result<Task, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok(), "任务应该可以反序列化");
}

#[test]
fn test_paginated_response_from_items() {
    let items = vec!["item1", "item2", "item3", "item4", "item5"];
    let response = rapidcron::types::PaginatedResponse::from_items(items, 1, 2);

    assert_eq!(response.total, 5);
    assert_eq!(response.page, 1);
    assert_eq!(response.page_size, 2);
    assert_eq!(response.total_pages, 3);
    assert_eq!(response.items.len(), 2);
}

#[test]
fn test_api_response_success() {
    let data = "test-data";
    let response = rapidcron::types::ApiResponse::success(data);

    assert!(response.success);
    assert!(response.data.is_some());
    assert!(response.message.is_none());
}
