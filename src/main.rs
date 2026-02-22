fn main() {
    if let Err(err) = lt::cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
