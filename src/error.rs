use thiserror::Error;
use crawler_template::CrawlerErr;

/// 主程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("File processing error: {0}")]
    #[allow(dead_code)]
    FileProcessing(String),
    
    #[error("Configuration error: {0}")]
    #[allow(dead_code)]
    Configuration(String),
    
    #[error("Movie data not found: {0}")]
    MovieDataNotFound(String),
    
    #[error("Movie data quality too low: {0}")]
    #[allow(dead_code)]
    MovieDataQualityTooLow(String),
    
    #[error("Template error: {0}")]
    Template(Box<CrawlerErr>),
    
    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    pub fn should_skip_processing(&self) -> bool {
        match self {
            AppError::MovieDataNotFound(_) | AppError::MovieDataQualityTooLow(_) => true,
            AppError::Template(crawler_err) => {
                if let CrawlerErr::Custom(msg) = crawler_err.as_ref() {
                    msg.starts_with("DATA_NOT_FOUND:")
                } else {
                    false
                }
            },
            _ => false,
        }
    }
    
    pub fn skip_reason(&self) -> Option<&str> {
        if self.should_skip_processing() {
            match self {
                AppError::MovieDataNotFound(_) => Some("影片数据不存在"),
                AppError::MovieDataQualityTooLow(_) => Some("数据质量过低"),
                AppError::Template(crawler_err) if matches!(crawler_err.as_ref(), CrawlerErr::Custom(msg) if msg.starts_with("DATA_NOT_FOUND:")) => Some("数据不存在"),
                _ => Some("未知原因"),
            }
        } else {
            None
        }
    }
}

impl From<CrawlerErr> for AppError {
    fn from(err: CrawlerErr) -> Self {
        match err {
            CrawlerErr::Custom(msg) if msg.starts_with("DATA_NOT_FOUND:") => {
                AppError::MovieDataNotFound(msg.strip_prefix("DATA_NOT_FOUND: ").unwrap_or(&msg).to_string())
            },
            other => AppError::Template(Box::new(other)),
        }
    }
}