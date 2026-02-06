fn main() {
    if let Err(err) = codexline::run() {
        eprintln!("codexline: {err:#}");
        std::process::exit(1);
    }
}
