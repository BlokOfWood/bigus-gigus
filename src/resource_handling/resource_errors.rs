#[derive(Debug)]
pub enum ResourceReferenceError {
    NotFound,
    TypeMismatch
}

#[derive(Debug)]
pub enum ResourceLoadError {
    NotFound,
    UnableToLoad,
    UnknownFileType,
}