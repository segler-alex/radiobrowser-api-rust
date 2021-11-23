use fern::colors::{Color, ColoredLevelConfig};
use std::io;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
struct StructuredLog {
    timestamp: String,
    level: String,
    target: String,
    message: String,
}

pub fn setup_logger(verbosity: usize, log_dir: &str, json: bool) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    let log_file_path = format!("{}/main.log", log_dir);

    base_config = match verbosity {
        0 => base_config
            .level(log::LevelFilter::Error)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Error),
        1 => base_config
            .level(log::LevelFilter::Warn)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Warn),
        2 => base_config
            .level(log::LevelFilter::Info)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Info),
        3 => base_config
            .level(log::LevelFilter::Info)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Debug),
        _4_or_more => base_config
            .level(log::LevelFilter::Info)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Trace),
    };

    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::BrightWhite)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let file_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            if json {
                let line = StructuredLog {
                    timestamp: chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S,%f")
                        .to_string(),
                    level: record.level().to_string(),
                    target: record.target().to_string(),
                    message: message.to_string(),
                };
                let line_str = serde_json::to_string(&line);
                match line_str {
                    Ok(line_str) => out.finish(format_args!("{}", line_str)),
                    Err(err) => out.finish(format_args!("Unable to encode log to JSON: {}", err)),
                }
            } else {
                out.finish(format_args!(
                    "{} {} {} {}",
                    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S,%f"),
                    record.level(),
                    record.target(),
                    message
                ))
            }
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(log_file_path)?);

    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            if json {
                let line = StructuredLog {
                    timestamp: chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S,%f")
                        .to_string(),
                    level: record.level().to_string(),
                    target: record.target().to_string(),
                    message: message.to_string(),
                };
                let line_str = serde_json::to_string(&line);
                match line_str {
                    Ok(line_str) => out.finish(format_args!("{}", line_str)),
                    Err(err) => out.finish(format_args!("Unable to encode log to JSON: {}", err)),
                }
            } else {
                out.finish(format_args!(
                    "{} {} {} {}",
                    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S,%f"),
                    colors_line.color(record.level()),
                    record.target(),
                    message
                ));
            }
        })
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;
    Ok(())
}
