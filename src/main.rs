mod args;
mod config;

use std::path::Path;

use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
    WriteMode,
};
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    log_init().unwrap();
    let start_param = args::StartParam::from_args_safe();
    let config = match config::AppConfig::new(start_param.unwrap().config_file) {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return;
        }
    };
}

fn log_init() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    let log_location = Path::new("./log");
    #[cfg(not(debug_assertions))]
    let log_location = Path::new("/log");
    if !log_location.exists() {
        std::fs::create_dir_all(&log_location)?;
    }
    let file_spec = FileSpec::default().directory(log_location);

    let _ = Logger::try_with_str("info,pago_mqtt=error,paho_mqtt_c=error")?
        .write_mode(WriteMode::SupportCapture)
        .log_to_file(file_spec)
        .duplicate_to_stderr(Duplicate::All)
        .format_for_stderr(colored_detailed_format)
        .format_for_stdout(colored_detailed_format)
        //https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
        .set_palette(String::from("b196;208;28;7;8"))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .start()?;
    Ok(())
}


