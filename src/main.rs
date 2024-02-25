use one_billion_crabs::process_file;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args.get(1).expect("No file provided");

    process_file(filename).expect("Error processing file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data() {
        let files = std::fs::read_dir("test-data").unwrap();

        for file in files {
            let file = file.unwrap();
            let path = file.path();
            let filename = path.to_str().unwrap();
            process_file(filename).expect("Error processing file");
        }
    }
}
