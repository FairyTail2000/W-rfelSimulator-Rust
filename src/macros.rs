/**
A macro to println debug info!
*/
#[macro_export]
macro_rules! dbgprintln {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        {
	        let formatted = format!($($arg)*);
			println!("{} {}: {}", file!(), line!(), formatted)
        }
    })
}

/**
A macro to print debug info!
*/
#[macro_export]
macro_rules! dbgprint {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        {
	        let formatted = format!($($arg)*);
			print!("{} {}: {}", file!(), line!(), formatted)
        }
    })
}