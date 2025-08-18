use std::path::Path;
use time::UtcOffset;
use time::macros::format_description;
use tracing::Level;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter, Registry, filter::LevelFilter, fmt::time::OffsetTime, layer::SubscriberExt,
    prelude::*, util::SubscriberInitExt,
};

pub fn local_offset() -> UtcOffset {
    UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)
}

/// 初始化日志系统，支持异步滚动文件日志
///
/// - `log_dir`: 日志目录，若为 None，则不输出文件日志
/// - `console_level`: 控制台日志等级
/// - `file_level`: 文件日志等级
pub fn init_logger<P: AsRef<Path>>(
    log_dir: Option<P>,
    console_level: Level,
    file_level: Level,
) -> Option<WorkerGuard> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    let offset = local_offset();
    let timer = OffsetTime::new(
        offset,
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    );

    // 简洁控制台日志，给用户看的，关闭源码路径和行号
    let console_layer = tracing_subscriber::fmt::layer()
        .with_timer(OffsetTime::new(offset, format_description!("")))
        .with_level(true)
        .with_target(false)
        .with_line_number(false)
        .with_file(false)
        .with_ansi(true)
        .compact()
        .with_filter(LevelFilter::from_level(console_level));

    // 详细文件日志，存储开发者查看用
    let file_layer_and_guard = log_dir.map(|dir| {
        let file_appender = tracing_appender::rolling::daily(dir, "app.log");
        let (non_blocking_writer, guard) = non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_timer(timer)
            .with_level(true)
            .with_target(true)
            .with_line_number(true)
            .with_file(true)
            .with_ansi(false)
            .with_writer(non_blocking_writer)
            .compact()
            .with_filter(LevelFilter::from_level(file_level));
        (file_layer, guard)
    });

    match file_layer_and_guard {
        Some((file_layer, guard)) => {
            Registry::default().with(env_filter).with(console_layer).with(file_layer).init();
            Some(guard)
        }
        None => {
            Registry::default().with(env_filter).with(console_layer).init();
            None
        }
    }
}

/// 简单控制台日志初始化
pub fn init_simple_logger(console_level: Level) {
    init_logger::<&str>(None, console_level, Level::ERROR);
}

//
// --- 统一日志宏定义部分 ---
//

#[macro_export]
macro_rules! log_msg_inner {
    (info, $($arg:tt)*) => {
        $crate::tracing::info!(
            "{} {}",
            $crate::console::Emoji("ℹ️", "i"),
            $crate::console::style(format!($($arg)*)).green()
        );
    };
    (warn, $($arg:tt)*) => {
        $crate::tracing::warn!(
            "{} {}",
            $crate::console::Emoji("⚠️", "!"),
            $crate::console::style(format!($($arg)*)).yellow()
        );
    };
    (error, $($arg:tt)*) => {
        $crate::tracing::error!(
            "{} {}",
            $crate::console::Emoji("✖️", "x"),
            $crate::console::style(format!($($arg)*)).red()
        );
    };
    (debug, $($arg:tt)*) => {
        $crate::tracing::debug!(
            "{} {}",
            $crate::console::Emoji("🐞", "D"),
            $crate::console::style(format!($($arg)*)).blue()
        );
    };
}

#[macro_export]
macro_rules! output {
    // 标题行输出（带 emoji）
    (title: $title:expr) => {{
        println!("\n{}", $title);
    }};

    // 普通提示行输出，带缩进
    (line: $($arg:tt)*) => {{
        println!("  {}", format!($($arg)*));
    }};

    // 空行
    (space) => {{
        println!();
    }};

    // 默认值提示
    (prompt: $label:expr, default = $default:expr) => {{
        println!("  {} [default: {}]", $label, $default);
    }};

    // 选项展示
    (options: $($opt:expr),+ $(,)?) => {{
        println!("  Options:");
        $(
            println!("    → {}", $opt);
        )+
    }};

    (error: $($arg:tt)*) => {{
        use $crate::console::style;
        let msg = format!($($arg)*);
        println!("  {} {}", style("✖").red().bold(), style(msg).red().bold());
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log_msg_inner!(info, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log_msg_inner!(warn, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log_msg_inner!(error, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::log_msg_inner!(debug, $($arg)*);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    use tracing::Level;

    #[test]
    fn test_logger_with_file_and_console() {
        use std::{fs, path::PathBuf, thread, time::Duration};
        use tempfile::tempdir;
        use time::{OffsetDateTime, format_description};

        let tmp_dir = tempdir().expect("failed to create temp dir");
        println!("temp dir path: {:?}", tmp_dir.path());

        // 初始化日志系统，带文件日志和控制台日志
        let _guard = init_logger(Some(tmp_dir.path()), Level::DEBUG, Level::DEBUG);

        // 发送各种日志
        log_info!("test_logger: info message");
        log_warn!("test_logger: warn message");
        log_error!("test_logger: error message");
        log_debug!("test_logger: debug message");

        // 等待异步写入完成
        thread::sleep(Duration::from_millis(1000));

        // 读取日志文件
        let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let fmt = format_description::parse("[year]-[month]-[day]").unwrap();
        let date_str = now.format(&fmt).unwrap();

        let mut log_file_path = PathBuf::from(tmp_dir.path());
        log_file_path.push(format!("app.log.{}", date_str));

        println!("Reading log file: {:?}", log_file_path);

        let content = fs::read_to_string(&log_file_path).expect("failed to read log file");
        println!("Log file content:\n{}", content);

        // 你也可以在这里断言日志内容包含特定信息，比如
        assert!(content.contains("info message"));
        assert!(content.contains("warn message"));
        assert!(content.contains("error message"));
    }
}
