fn main() {
    // Take in file as argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("No file provided");

    // Read file
    let contents =
        std::fs::read_to_string(filename).expect("Something went wrong reading the file");

    // Split file into lines
    let lines: Vec<&str> = contents.split('\n').collect();

    println!("Line count: {}", lines.len());
}
