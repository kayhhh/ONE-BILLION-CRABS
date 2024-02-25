use one_billion_crabs::process_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("No file provided");

    let out = process_file(filename).expect("Error processing file");
    println!("Output written to {}", out);
}
