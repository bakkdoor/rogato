pub fn print_error<E: std::fmt::Debug>(error: E) -> E {
    eprintln!("Error doing DB stuff: {:?}", error);
    error
}
