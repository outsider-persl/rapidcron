// build.rs 放在项目根目录
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置tonic代码生成规则
    tonic_build::configure()
        .out_dir("src/grpc/proto/") // 生成的代码输出到proto目录
        .compile(
            &["src/grpc/proto/task_scheduler.proto"], // 源proto文件
            &["src/grpc/proto/"],                      // include路径
        )?;

    // 告诉Cargo如果proto文件修改，需要重新编译
    println!("cargo:rerun-if-changed=src/grpc/proto/task_scheduler.proto");
    Ok(())
}