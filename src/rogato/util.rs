pub fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error: {:?}", error);
    error
}
