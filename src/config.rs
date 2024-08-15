pub struct Config {
    pub scale_factor: u32,
}

impl Config {
    pub fn new(scale_factor: u32) -> Self {
        Config { scale_factor }
    }
}
