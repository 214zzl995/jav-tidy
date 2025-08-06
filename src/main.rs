mod args;
mod config;
mod crawler;
mod error;
mod file;
mod file_organizer;
mod image_manager;
mod nfo;
mod nfo_generator;
mod parser;
mod template_parser;
mod translator;

use std::path::Path;

use std::result::Result::{Ok, Err};
use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
    WriteMode,
};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use structopt::{StructOpt, clap::ErrorKind};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arg = match args::StartParam::from_args_safe() {
        Ok(arg) => {
            println!("JAV-Tidy-RS 启动中...");
            println!("配置文件: {}", arg.config_file.display());
            println!("日志目录: {}", arg.log_location.display());
            println!("模板目录: {}", arg.template_location.display());
            arg
        },
        Err(e) => {
            // 如果是帮助或版本信息，正常退出
            if e.kind == ErrorKind::HelpDisplayed 
                || e.kind == ErrorKind::VersionDisplayed {
                println!("{}", e.message);
                std::process::exit(0);
            } else {
                eprintln!("参数解析错误: {}", e);
                std::process::exit(1);
            }
        }
    };
    
    println!("初始化日志系统...");
    let multi_progress = log_init(&arg.log_location).unwrap();

    println!("加载应用配置...");
    let config = config::AppConfig::new(&arg.config_file)?;
    log::info!("应用配置加载完成");
    log::info!("输入目录: {}", config.input_dir.display());
    log::info!("输出目录: {}", config.get_output_dir().display());
    log::info!("支持的文件类型: {:?}", config.get_migrate_files_ext());

    println!("创建文件处理通道...");
    let (file_tx, file_rx) = tokio::sync::mpsc::channel(8);
    log::info!("文件处理通道创建完成，通道容量: 8");
    
    println!("初始化文件监控系统...");
    let _source_notify = file::initial(&config, file_tx).await?;

    println!("初始化爬虫系统...");
    crawler::initial(&arg.template_location, &config, file_rx, multi_progress)?;

    println!("JAV-Tidy-RS 初始化完成，开始监控文件...");
    log::info!("JAV-Tidy-RS 已完全启动，等待文件处理");

    // 保持主线程运行
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        log::debug!("主线程保活检查");
    }
}

fn log_init(log_location: &Path) -> anyhow::Result<MultiProgress> {
    if log_location.is_file() {
        return Err(anyhow::anyhow!("log file is a file, not a directory"));
    } else if log_location.is_dir() {
        if !log_location.exists() {
            std::fs::create_dir_all(log_location)?;
        }
    } else {
        return Err(anyhow::anyhow!("log file is not a file or directory"));
    }

    let file_spec = FileSpec::default().directory(log_location);

    let (logger, _) = Logger::try_with_str("info,pago_mqtt=error,paho_mqtt_c=error")?
        .write_mode(WriteMode::SupportCapture)
        .log_to_file(file_spec)
        .duplicate_to_stderr(Duplicate::All)
        .format_for_stderr(colored_detailed_format)
        .format_for_stdout(colored_detailed_format)
        .set_palette(String::from("b196;208;28;7;8"))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .build()?;

    let multi = MultiProgress::new();

    LogWrapper::new(multi.clone(), logger).try_init().unwrap();

    Ok(multi)
}
