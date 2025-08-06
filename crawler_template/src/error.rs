use thiserror::Error;

// Define crawler errors
#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)] // pest::error::Error is inherently large
pub enum CrawlerErr {
    #[error("No data found for node: {node}")]
    NotFound { node: &'static str },
    #[error("IO error: {msg}")]
    IOError { msg: String },
    #[error("Other error: {0}")]
    OtherError(String),
    #[error("Template not found")]
    TempNotFound,
    #[error("Template internal information error, error cause: {0}")]
    YamlTempFormatError(#[from] serde_yaml::Error),
    #[error("Selector parse failure , {0}")]
    SelectorError(String),
    #[error("{0}")]
    ScriptParseError(#[from] pest::error::Error<crate::script::Rule>),
    #[error("{0}")]
    RegexParseError(#[from] regex::Error),
    #[error("Variable '${{{0}}}' not yet initialized")]
    DynNotYetInitialised(String),
    #[error("Variable '${{{0}}}' has no valid data")]
    DynNoValidData(String),
    #[error("Variable '${{{0}}}' does not support multiple results, currently got: {1}")]
    DynMultipleResults(String, String),
    #[error("parent({0}) Parent node overflow, current highest parent node: {1}")]
    ParentNodeOverflow(usize, usize),
    #[error("prev({0}) Previous sibling node overflow, current highest prev node: {1}")]
    PrevNodeOverflow(usize, usize),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Node '{0}' got incorrect number of values: {1}")]
    InvalidValueCount(String, usize),
    #[error("Field '{0}' not found in the template")]
    FieldNotFound(String),
    #[error("Field '{0}' parse error: {1}")]
    ParseError(String, String),
    #[error("Invalid transform rule")]
    InvalidTransformRule,
    #[error("Missing index parameter")]
    MissingIndex,
    #[error("Unsupported transform rule")]
    UnsupportedTransformRule,
    #[error("Unsupported selector rule")]
    UnsupportedSelectorRule,
    #[error("Crawler script cannot use character processing functions in isolation")]
    CharProcessAlone,
    #[error("Entry point environment variable '${{{0}}}' has multiple parameter values")]
    MultipleEntrypointParameterError(String),

    #[error("Data not found: {0}")]
    DataNotFound(String),
    
    #[error("Required field validation failed: {0}")]
    RequiredFieldValidationFailed(String),
    
    #[error("Custom error: {0}")]
    Custom(String),

    #[error("{0}")]
    CrawlerParseError(#[from] CrawlerParseError),
}

#[derive(Debug, Error)]
pub enum CrawlerParseError {
    #[error("Parse error: {0}")]
    MissingField(&'static str),
    #[error("Parse error: {0}")]
    ConversionFailed(&'static str),
    #[error("Parse error: {0}")]
    EmptyVector(&'static str),
}
