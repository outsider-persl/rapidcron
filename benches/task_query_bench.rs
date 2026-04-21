use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rapidcron::types::{PaginatedResponse, Task, TaskInstance, TaskPayload, TaskStatus, TaskType};
use mongodb::bson::oid::ObjectId;
use chrono::Utc;

fn bench_paginated_response_from_items_small(c: &mut Criterion) {
    let items: Vec<String> = (0..20).map(|i| format!("item-{}", i)).collect();

    c.bench_function("paginated_response_small", |b| {
        b.iter(|| PaginatedResponse::from_items(black_box(items.clone()), 1, 10));
    });
}

fn bench_paginated_response_from_items_medium(c: &mut Criterion) {
    let items: Vec<String> = (0..100).map(|i| format!("item-{}", i)).collect();

    c.bench_function("paginated_response_medium", |b| {
        b.iter(|| PaginatedResponse::from_items(black_box(items.clone()), 1, 20));
    });
}

fn bench_paginated_response_from_items_large(c: &mut Criterion) {
    let items: Vec<String> = (0..1000).map(|i| format!("item-{}", i)).collect();

    c.bench_function("paginated_response_large", |b| {
        b.iter(|| PaginatedResponse::from_items(black_box(items.clone()), 1, 50));
    });
}

fn bench_task_instance_serialization(c: &mut Criterion) {
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
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    c.bench_function("task_instance_serialization", |b| {
        b.iter(|| serde_json::to_string(black_box(&instance)));
    });
}

fn bench_task_instance_deserialization(c: &mut Criterion) {
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
        triggered_by: rapidcron::types::TriggeredBy::Scheduler,
        created_at: Utc::now(),
    };

    let serialized = serde_json::to_string(&instance).unwrap();

    c.bench_function("task_instance_deserialization", |b| {
        b.iter(|| {
            let _: Result<TaskInstance, _> = serde_json::from_str(black_box(&serialized));
        });
    });
}

fn bench_task_serialization(c: &mut Criterion) {
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

fn bench_parse_object_id(c: &mut Criterion) {
    let id_str = "507f1f77bcf86cd799439011";

    c.bench_function("parse_object_id", |b| {
        b.iter(|| rapidcron::types::parse_object_id(black_box(id_str)));
    });
}

fn bench_parse_object_ids_batch(c: &mut Criterion) {
    let ids: Vec<String> = (0..100)
        .map(|_| "507f1f77bcf86cd799439011".to_string())
        .collect();

    c.bench_function("parse_object_ids_batch", |b| {
        b.iter(|| rapidcron::types::parse_object_ids(black_box(&ids)));
    });
}

fn bench_api_response_creation(c: &mut Criterion) {
    c.bench_function("api_response_creation", |b| {
        b.iter(|| rapidcron::types::ApiResponse::success(black_box("test-data")));
    });
}

criterion_group!(
    benches,
    bench_paginated_response_from_items_small,
    bench_paginated_response_from_items_medium,
    bench_paginated_response_from_items_large,
    bench_task_instance_serialization,
    bench_task_instance_deserialization,
    bench_task_serialization,
    bench_task_deserialization,
    bench_parse_object_id,
    bench_parse_object_ids_batch,
    bench_api_response_creation
);
criterion_main!(benches);
