fn main() {
    if let Err(error) = luu_gui::backend::run() {
        eprintln!("ERROR: {error}");
        std::process::exit(1);
    }
}
