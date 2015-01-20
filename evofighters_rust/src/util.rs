// TODO: add the stars to different debug statements
#[macro_export]
macro_rules! print1 {
    ($($arg:tt)*) => (
        if cfg!(any(v1,v2,v3)){
            println!($($arg)*);
        })
}

#[macro_export]
macro_rules! print2 {
    ($($arg:tt)*) => (
        if cfg!(any(v2,v3)){
            println!($($arg)*);
        })
}

#[macro_export]
macro_rules! print3 {
    ($($arg:tt)*) => (
        if cfg!(v3){
            println!($($arg)*);
        })
}
