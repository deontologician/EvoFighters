// TODO: add the stars to different debug statements
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => (
        if cfg!(feature = "log_trace") {
            println!($($arg)*);
        })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (
        if cfg!(feature = "log_trace") ||
            cfg!(feature = "log_debug") {
            println!($($arg)*);
        })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (
        if cfg!(feature = "log_info") ||
            cfg!(feature = "log_debug") ||
            cfg!(feature = "log_trace") {
            println!($($arg)*);
        })
}
