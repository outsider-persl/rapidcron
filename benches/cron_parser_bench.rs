use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rapidcron::scheduler::cron_parser::CronParser;
use chrono::{Duration, Utc};

fn bench_cron_parser_new(c: &mut Criterion) {
    let expressions = vec![
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

    let mut group = c.benchmark_group("cron_parser_new");

    for expr in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(expr), expr, |b, e| {
            b.iter(|| CronParser::new(black_box(e)));
        });
    }

    group.finish();
}

fn bench_cron_parser_next_triggers_simple(c: &mut Criterion) {
    let parser = CronParser::new("0/5 * * * * *").unwrap();
    let start = Utc::now();
    let end = start + Duration::seconds(60);

    c.bench_function("next_triggers_simple", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_next_triggers_complex(c: &mut Criterion) {
    let parser = CronParser::new("0,15,30,45 * * * * *").unwrap();
    let start = Utc::now();
    let end = start + Duration::minutes(5);

    c.bench_function("next_triggers_complex", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_next_triggers_every_second(c: &mut Criterion) {
    let parser = CronParser::new("* * * * * *").unwrap();
    let start = Utc::now();
    let end = start + Duration::seconds(10);

    c.bench_function("next_triggers_every_second", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_next_triggers_hourly(c: &mut Criterion) {
    let parser = CronParser::new("0 0 * * * *").unwrap();
    let start = Utc::now();
    let end = start + Duration::hours(25);

    c.bench_function("next_triggers_hourly", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_next_triggers_daily(c: &mut Criterion) {
    let parser = CronParser::new("0 0 0 * * *").unwrap();
    let start = Utc::now();
    let end = start + Duration::days(7);

    c.bench_function("next_triggers_daily", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_next_triggers_weekly(c: &mut Criterion) {
    let parser = CronParser::new("0 0 0 * * 1").unwrap();
    let start = Utc::now();
    let end = start + Duration::weeks(4);

    c.bench_function("next_triggers_weekly", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_next_triggers_monthly(c: &mut Criterion) {
    let parser = CronParser::new("0 0 0 1 * *").unwrap();
    let start = Utc::now();
    let end = start + Duration::days(90);

    c.bench_function("next_triggers_monthly", |b| {
        b.iter(|| parser.next_triggers_in_window(black_box(start), black_box(end)));
    });
}

fn bench_cron_parser_batch_parsing(c: &mut Criterion) {
    let expressions = vec![
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

    c.bench_function("batch_parsing", |b| {
        b.iter(|| {
            for expr in &expressions {
                let _ = CronParser::new(black_box(expr));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_cron_parser_new,
    bench_cron_parser_next_triggers_simple,
    bench_cron_parser_next_triggers_complex,
    bench_cron_parser_next_triggers_every_second,
    bench_cron_parser_next_triggers_hourly,
    bench_cron_parser_next_triggers_daily,
    bench_cron_parser_next_triggers_weekly,
    bench_cron_parser_next_triggers_monthly,
    bench_cron_parser_batch_parsing
);
criterion_main!(benches);
