use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rapidcron::types::{CreateTaskRequest, TaskPayload, TaskType};
use chrono::Utc;

fn bench_create_task_request_command(c: &mut Criterion) {
    let request = CreateTaskRequest {
        name: "test-task".to_string(),
        description: Some("Test task".to_string()),
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'Hello World'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    c.bench_function("create_task_command", |b| {
        b.iter(|| request.to_task());
    });
}

fn bench_create_task_request_http(c: &mut Criterion) {
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

    c.bench_function("create_task_http", |b| {
        b.iter(|| request.to_task());
    });
}

fn bench_create_task_request_with_dependencies(c: &mut Criterion) {
    let request = CreateTaskRequest {
        name: "task-with-deps".to_string(),
        description: Some("Task with dependencies".to_string()),
        dependency_ids: vec![
            "507f1f77bcf86cd799439011".to_string(),
            "507f1f77bcf86cd799439012".to_string(),
            "507f1f77bcf86cd799439013".to_string(),
        ],
        task_type: Some("command".to_string()),
        schedule: "0/10 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'test'".to_string()),
        url: None,
        timeout_seconds: Some(60),
        max_retries: Some(5),
    };

    c.bench_function("create_task_with_dependencies", |b| {
        b.iter(|| request.to_task());
    });
}

fn bench_create_task_request_complex_schedule(c: &mut Criterion) {
    let request = CreateTaskRequest {
        name: "complex-task".to_string(),
        description: Some("Complex schedule task".to_string()),
        dependency_ids: vec![],
        task_type: Some("command".to_string()),
        schedule: "0,15,30,45 * * * * *".to_string(),
        enabled: true,
        command: Some("echo 'complex'".to_string()),
        url: None,
        timeout_seconds: Some(30),
        max_retries: Some(3),
    };

    c.bench_function("create_task_complex_schedule", |b| {
        b.iter(|| request.to_task());
    });
}

fn bench_create_task_request_batch(c: &mut Criterion) {
    let requests: Vec<CreateTaskRequest> = (0..100)
        .map(|i| CreateTaskRequest {
            name: format!("task-{}", i),
            description: Some(format!("Test task {}", i)),
            dependency_ids: vec![],
            task_type: Some("command".to_string()),
            schedule: "0/5 * * * * *".to_string(),
            enabled: true,
            command: Some(format!("echo 'Task {}'", i)),
            url: None,
            timeout_seconds: Some(30),
            max_retries: Some(3),
        })
        .collect();

    c.bench_function("create_task_batch", |b| {
        b.iter(|| {
            for request in &requests {
                let _ = request.to_task();
            }
        });
    });
}

fn bench_task_serialization(c: &mut Criterion) {
    use rapidcron::types::Task;
    use mongodb::bson::oid::ObjectId;

    let task = Task {
        id: Some(ObjectId::new()),
        name: "test-task".to_string(),
        description: Some("Test task".to_string()),
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'Hello World'".to_string(),
            timeout_seconds: Some(30),
        },
        timeout_seconds: Some(30),
        max_retries: Some(3),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    c.bench_function("task_serialization", |b| {
        b.iter(|| serde_json::to_string(black_box(&task)));
    });
}

fn bench_task_deserialization(c: &mut Criterion) {
    use rapidcron::types::Task;
    use mongodb::bson::oid::ObjectId;

    let task = Task {
        id: Some(ObjectId::new()),
        name: "test-task".to_string(),
        description: Some("Test task".to_string()),
        dependency_ids: vec![],
        task_type: TaskType::Command,
        schedule: "0/5 * * * * *".to_string(),
        enabled: true,
        payload: TaskPayload::Command {
            command: "echo 'Hello World'".to_string(),
            timeout_seconds: Some(30),
        },
        timeout_seconds: Some(30),
        max_retries: Some(3),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let serialized = serde_json::to_string(&task).unwrap();

    c.bench_function("task_deserialization", |b| {
        b.iter(|| {
            let _: Result<Task, _> = serde_json::from_str(black_box(&serialized));
        });
    });
}

criterion_group!(
    benches,
    bench_create_task_request_command,
    bench_create_task_request_http,
    bench_create_task_request_with_dependencies,
    bench_create_task_request_complex_schedule,
    bench_create_task_request_batch,
    bench_task_serialization,
    bench_task_deserialization
);
criterion_main!(benches);
