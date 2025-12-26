pub trait Component {
    fn get_type_fingerprint() -> &'static str;
}