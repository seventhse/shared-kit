use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_level(true)
        .with_line_number(true)
        .with_file(true)
        .compact();

    tracing_subscriber::registry().with(env_filter).with(fmt_layer).init();
}

#[macro_export]
macro_rules! info_msg {
    ($($arg:tt)*) => {
        println!(
            "{} {}",
            console::Emoji("ℹ️", "i"),
            console::style(format!($($arg)*)).green()
        );
    };
}

#[macro_export]
macro_rules! warn_msg {
    ($($arg:tt)*) => {
        println!(
            "{} {}",
            console::Emoji("⚠️", "!"),
            console::style(format!($($arg)*)).yellow()
        );
    };
}

#[macro_export]
macro_rules! error_msg {
    ($($arg:tt)*) => {
        eprintln!(
            "{} {}",
            console::Emoji("✖️", "x"),
            console::style(format!($($arg)*)).red()
        )
    };
}
