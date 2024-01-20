#[derive(Clone)]
pub struct Config {
    pub vus: usize,
    pub duration: usize,
    pub spawn_rate: usize,
    pub text_width: usize,
    pub url: String,
}

#[derive(Debug)]
pub struct WorkerState {
    pub rank: usize,
    pub ite: usize,
    pub is_first: bool,
    pub begin: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub content: String,
}
