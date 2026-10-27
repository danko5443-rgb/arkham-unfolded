fn main() {
    if let Err(error) = arkham_unfolded_experiment::tabletop::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
