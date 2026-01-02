pub trait Component: Clone {
    fn get_type_fingerprint() -> &'static str;
}