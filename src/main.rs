use one_billion_crabs::process_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("No file provided");

    process_file(filename).expect("Error processing file");
}
