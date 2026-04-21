#!/usr/bin/env bash

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

LOG_DIR="$PROJECT_ROOT/logs"
mkdir -p "$LOG_DIR"

timestamp() {
  # 格式: yyyy-mm-dd-HH:MM:SS
  date +"%Y-%m-%d-%H:%M:%S"
}

sanitize_name() {
  # Replace spaces and slashes to keep filename safe
  echo "$1" | tr ' /' '__'
}

select_test_type() {
  echo "请选择测试类型:"
  echo "1) 单元测试"
  echo "2) 集成测试"
  echo "3) 基准测试"
  read -r -p "输入选项编号: " TEST_TYPE_CHOICE

  case "$TEST_TYPE_CHOICE" in
    1)
      TEST_TYPE="unit"
      ;;
    2)
      TEST_TYPE="integration"
      ;;
    3)
      TEST_TYPE="bench"
      ;;
    *)
      echo "无效选项，退出。"
      exit 1
      ;;
  esac
}

select_unit_or_integration_scope() {
  local kind="$1" # unit or integration

  echo "请选择${kind}测试范围:"
  echo "1) 全部"
  echo "2) 从列表中选择模块"
  read -r -p "输入选项编号: " SCOPE_CHOICE

  case "$SCOPE_CHOICE" in
    1)
      TEST_SCOPE="all"
      ;;
    2)
      TEST_SCOPE="module"
      if [ "$kind" = "unit" ]; then
        echo "请选择单元测试模块:"
        echo "1) cron_parser"
        echo "2) task_management"
        echo "3) task_execution"
        echo "4) retry_logic"
        read -r -p "输入选项编号: " MODULE_CHOICE
        case "$MODULE_CHOICE" in
          1) TEST_MODULE="cron_parser" ;;
          2) TEST_MODULE="task_management" ;;
          3) TEST_MODULE="task_execution" ;;
          4) TEST_MODULE="retry_logic" ;;
          *)
            echo "无效选项，退出。"
            exit 1
            ;;
        esac
      else
        echo "请选择集成测试模块(对应 tests/*.rs 文件名):"
        echo "1) cron_parser_integration"
        echo "2) task_management"
        echo "3) task_execution"
        echo "4) retry_logic"
        read -r -p "输入选项编号: " MODULE_CHOICE
        case "$MODULE_CHOICE" in
          1) TEST_MODULE="cron_parser_integration" ;;
          2) TEST_MODULE="task_management" ;;
          3) TEST_MODULE="task_execution" ;;
          4) TEST_MODULE="retry_logic" ;;
          *)
            echo "无效选项，退出。"
            exit 1
            ;;
        esac
      fi
      ;;
    *)
      echo "无效选项，退出。"
      exit 1
      ;;
  esac
}

select_bench_target() {
  echo "请选择基准测试目标:"
  echo "1) 全部基准测试"
  echo "2) cron_parser_bench"
  echo "3) task_creation_bench"
  echo "4) task_query_bench"
  echo "5) retry_calculation_bench"
  read -r -p "输入选项编号: " BENCH_SCOPE_CHOICE

  case "$BENCH_SCOPE_CHOICE" in
    1)
      BENCH_SCOPE="all"
      BENCH_NAME=""
      ;;
    2)
      BENCH_SCOPE="single"
      BENCH_NAME="cron_parser_bench"
      ;;
    3)
      BENCH_SCOPE="single"
      BENCH_NAME="task_creation_bench"
      ;;
    4)
      BENCH_SCOPE="single"
      BENCH_NAME="task_query_bench"
      ;;
    5)
      BENCH_SCOPE="single"
      BENCH_NAME="retry_calculation_bench"
      ;;
    *)
      echo "无效选项，退出。"
      exit 1
      ;;
  esac
}

select_bench_mode() {
  echo "请选择基准测试模式:"
  echo "1) 快速模式"
  echo "2) 标准模式"
  echo "3) 精准模式"
  read -r -p "输入选项编号: " BENCH_MODE_CHOICE

  case "$BENCH_MODE_CHOICE" in
    1)
      BENCH_MODE="fast"
      SAMPLE_SIZE=20
      WARM_UP=1
      ;;
    2)
      BENCH_MODE="standard"
      SAMPLE_SIZE=80
      WARM_UP=3
      ;;
    3)
      BENCH_MODE="precise"
      SAMPLE_SIZE=200
      WARM_UP=8
      ;;
    *)
      echo "无效选项，退出。"
      exit 1
      ;;
  esac
}

run_tests() {
  local description
  local log_name_prefix
  local cmd

  if [ "$TEST_TYPE" = "unit" ]; then
    if [ "$TEST_SCOPE" = "all" ]; then
      description="单元测试-全部"
      log_name_prefix="unit-all"
      cmd=(cargo test)
    else
      description="单元测试-${TEST_MODULE}"
      log_name_prefix="unit-${TEST_MODULE}"
      cmd=(cargo test "$TEST_MODULE")
    fi
  elif [ "$TEST_TYPE" = "integration" ]; then
    if [ "$TEST_SCOPE" = "all" ]; then
      description="集成测试-全部"
       log_name_prefix="integration-all"
      cmd=(cargo test --tests)
    else
      description="集成测试-${TEST_MODULE}"
      log_name_prefix="integration-${TEST_MODULE}"
      cmd=(cargo test --test "$TEST_MODULE")
    fi
  else
    # bench
    if [ "$BENCH_SCOPE" = "all" ]; then
      description="基准测试-全部-${BENCH_MODE}"
      log_name_prefix="bench-all-${BENCH_MODE}"
      cmd=(cargo bench -- --sample-size "$SAMPLE_SIZE" --warm-up-time "$WARM_UP")
    else
      description="基准测试-${BENCH_NAME}-${BENCH_MODE}"
      log_name_prefix="bench-${BENCH_NAME}-${BENCH_MODE}"
      cmd=(cargo bench --bench "$BENCH_NAME" -- --sample-size "$SAMPLE_SIZE" --warm-up-time "$WARM_UP")
    fi
  fi

  local safe_name
  safe_name=$(sanitize_name "$log_name_prefix")
  local log_file="$LOG_DIR/${safe_name}-$(timestamp).log"

  echo "即将运行: ${cmd[*]}"
  echo "日志输出到: $log_file"
  echo "----------------------------------------" | tee "$log_file"
  echo "测试描述: $description" | tee -a "$log_file"
  echo "命令: ${cmd[*]}" | tee -a "$log_file"
  echo "开始时间: $(date)" | tee -a "$log_file"
  echo "----------------------------------------" | tee -a "$log_file"

  # 既在终端显示又写入日志
  "${cmd[@]}" 2>&1 | tee -a "$log_file"

  local exit_code=${PIPESTATUS[0]}
  echo "----------------------------------------" | tee -a "$log_file"
  echo "结束时间: $(date)" | tee -a "$log_file"
  echo "退出码: $exit_code" | tee -a "$log_file"

  exit "$exit_code"
}

select_test_type

case "$TEST_TYPE" in
  unit)
    select_unit_or_integration_scope "unit"
    ;;
  integration)
    select_unit_or_integration_scope "integration"
    ;;
  bench)
    select_bench_target
    select_bench_mode
    ;;
esac

run_tests

