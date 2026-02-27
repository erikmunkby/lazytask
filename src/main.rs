fn main() {
    if let Err(err) = lazytask::cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
