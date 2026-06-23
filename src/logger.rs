use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn init() {
    let mut builder = Builder::new();
    builder.format(|buf, record| {
        writeln!(
            buf,
            "[{} - {} - {}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });
    builder.filter(None, LevelFilter::Trace);
    let logger = builder.build();
    let _ = log::set_boxed_logger(Box::new(logger));
    log::set_max_level(LevelFilter::Info);
}

pub fn set_debug_level(level: LevelFilter) {
    log::set_max_level(level);
}
