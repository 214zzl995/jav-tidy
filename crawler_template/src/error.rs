use thiserror::Error;

//定义错误
#[derive(Error, Debug)]
pub enum CrawlerErr {
    #[error("未爬取到数据,节点为：{node}")]
    NotFound { node: &'static str },
    #[error("IO错误:{msg}")]
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
    #[error("${{0}} Not yet initialised")]
    DynNotYetInitialised(String),
    #[error("${{0}} No valid data on the results of the")]
    DynNoValidData(String),
    #[error("${{0}} Variables that do not support multiple results,Currently getting multiple results: {1}")]
    DynMultipleResults(String, String),
    #[error("parent({0}) Parent node overflow , Current highest parent node: {1}")]
    ParentNodeOverflow(usize, usize),
    #[error("prev({0}) Prev node overflow , Current highest prev node: {1}")]
    PrevNodeOverflow(usize, usize),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("The node {0} got an incorrect number of :{1}")]
    InvalidValueCount(String, usize),
    #[error("The field {0} not found in the tempalte")]
    FieldNotFound(String),
    #[error("The field {0} parse error: {1}")]
    ParseError(String, String),
    #[error("InvalidTransformRule")]
    InvalidTransformRule,
    #[error("MissingIndex")]
    MissingIndex,
    #[error("UnsupportedTransformRule")]
    UnsupportedTransformRule,
    #[error("UnsupportedSelectorRule")]
    UnsupportedSelectorRule,
    #[error("The crawler script cannot use character processing functions alone")]
    CharProcessAlone,
    #[error("Entry point environment variable ${{1}} has multiple parameter values.")]
    MultipleEntrypointParameterError(String),

    #[error("{0}")]
    CrawlerParseError(#[from] CrawlerParseError),
    #[error("{0}")]
    CustomError(#[from] Box<dyn std::error::Error>),
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
