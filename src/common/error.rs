//! 全局错误定义模块
//!
//! 定义 Cron 模块中出现的结构化错误类型，
//! 支持 gRPC、HTTP、SDK 上层进行统一返回处理。

use thiserror::Error;

/// Cron 调度相关错误类型
#[derive(Error, Debug)]
pub enum CronError {
    #[error("表达式语法错误: {0}")]
    SyntaxError(String),

    #[error("时间字段超出允许范围: {0}")]
    TimeRangeError(String),

    #[error("表达式字段数量错误: {0}")]
    FieldCountError(String),

    #[error("内部错误: {0}")]
    InternalError(String),
}
