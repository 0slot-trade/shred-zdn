#[macro_export]
macro_rules! same_dir {
    ($filename: expr) => {{
        let dir = std::path::Path::new(file!())
            .parent().unwrap(); 
        dir.join($filename) 
            .to_str().unwrap()
            .to_owned()
    }};
}

#[macro_export]
macro_rules! location {
    () => {
        format!("{}:{}", file!(), line!())
    };
}
