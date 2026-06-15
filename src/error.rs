#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ptree: path not found: `{0}`")]
    PathNotFound(String),

    #[error("ptree: cannot access `{0}`: {1}")]
    AccessPath(String, #[source] std::io::Error),

    #[error("ptree: failed to open directory `{0}`: {1}")]
    ReadDir(String, #[source] std::io::Error),

    #[error("ptree: failed to read directory entry in `{0}`: {1}")]
    ReadDirEntry(String, #[source] std::io::Error),

    #[error("ptree: failed to get file type of `{0}`: {1}")]
    GetFileType(String, #[source] std::io::Error),

    #[error("ptree: failed to get metadata of `{0}`: {1}")]
    GetMetadata(String, #[source] std::io::Error),

    #[error("ptree: failed to read .gitignore file `{0}`: {1}")]
    ReadGitignore(String, #[source] std::io::Error),

    #[error("ptree: failed to parse .gitignore file `{path}`: {detail}")]
    GitignoreParse { path: String, detail: String },

    #[error("ptree: failed to read symlink `{0}`: {1}")]
    ReadSymlink(String, #[source] std::io::Error),

    #[error("ptree: invalid glob pattern `{pattern}`: {detail}")]
    GlobParse { pattern: String, detail: String },

    #[error("ptree: config file not found: `{0}`")]
    ConfigNotFound(String),

    #[error("ptree: failed to read config file `{0}`: {1}")]
    ReadConfig(String, #[source] std::io::Error),

    #[error("ptree: failed to parse config file `{0}`: {1}")]
    ParseConfig(String, String),

    #[error("ptree: failed to encode JSON output: {0}")]
    JsonEncode(#[from] serde_json::Error),
}
