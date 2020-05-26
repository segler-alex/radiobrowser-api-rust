use fern::colors::{Color, ColoredLevelConfig};
use std::io;

pub fn setup_logger(verbosity: usize, log_dir: &str, _json: bool) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();

    let log_file_path = format!("{}/main.log", log_dir);

    base_config = match verbosity {
        0 => base_config
            .level(log::LevelFilter::Warn)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Warn),
        1 => base_config
            .level(log::LevelFilter::Info)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Info),
        2 => base_config
            .level(log::LevelFilter::Info)
            .level_for("radiobrowser_api_rust", log::LevelFilter::Debug),
        _3_or_more => base_config
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
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {} {} {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S,%f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(log_file_path)?);

    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {} {} {}",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S,%f"),
                colors_line.color(record.level()),
                record.target(),
                message
            ));
        })
        .chain(io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;
    Ok(())
}
