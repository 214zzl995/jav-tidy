mod args;
mod config;
mod crawler;
mod file;
mod file_organizer;
mod nfo;
mod nfo_generator;
mod parser;

use std::path::Path;

use anyhow::Ok;
use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
    WriteMode,
};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arg = args::StartParam::from_args_safe().unwrap();
    let multi_progress = log_init(&arg.log_location).unwrap();

    let config = config::AppConfig::new(&arg.config_file)?;

    let (file_tx, file_rx) = tokio::sync::mpsc::channel(8);
    let _source_notify = file::initial(&config, file_tx).await?;

    crawler::initial(&arg.template_location, &config, file_rx, multi_progress)?;

    Ok(())
}

fn log_init(log_location: &Path) -> anyhow::Result<MultiProgress> {
    if log_location.is_file() {
        return Err(anyhow::anyhow!("log file is a file, not a directory"));
    } else if log_location.is_dir() {
        if !log_location.exists() {
            std::fs::create_dir_all(&log_location)?;
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
