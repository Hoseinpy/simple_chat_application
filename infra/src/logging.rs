use std::io::Write;

use chrono::Local;

pub fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] [{}:{}]: {}",
                Local::now().format("%F %T"),
                record.level(),
                record.file().unwrap_or("NaN"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Info)
        .init();
}
