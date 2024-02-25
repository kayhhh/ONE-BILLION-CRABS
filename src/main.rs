use one_billion_crabs::process_file;

const MAX_THREADS: usize = 8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("No file provided");

    let mut num_threads = std::thread::available_parallelism().unwrap().get() - 1;

    if num_threads < 1 {
        num_threads = 1;
    }

    if num_threads > MAX_THREADS {
        num_threads = MAX_THREADS;
    }

    println!("Using {} threads", num_threads);

    let out = process_file(filename, num_threads).expect("Error processing file");
    println!("Output written to {}", out);
}
