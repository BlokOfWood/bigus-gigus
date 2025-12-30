#[derive(Debug)]
pub enum ResourceReferenceError {
    InvalidResourceHandle,
    TypeMismatch,
}

#[derive(Debug)]
pub enum ResourceLoadError {
    NotFound,
    UnableToLoad,
    UnknownFileType,
}
