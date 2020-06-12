use std::fs::File;

pub enum ApiResponse {
    Text(String, String),
    File(String, File),
    ServerError(String),
    NotFound,
    UnknownContentType,
    //ParameterError(String),
    Locked(String),
}
