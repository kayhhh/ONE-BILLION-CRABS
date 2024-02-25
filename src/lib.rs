use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::Write,
};

#[derive(Clone, Debug)]
struct WeatherStation {
    count: usize,
    max: f64,
    min: f64,
    total: f64,
}

impl Default for WeatherStation {
    fn default() -> Self {
        WeatherStation {
            count: 0,
            max: -std::f64::INFINITY,
            min: std::f64::INFINITY,
            total: 0.0,
        }
    }
}

const SIGNAL_END: &str = "END";

/// Process the given file and return the name of the output file.
pub fn process_file(path: &str, num_threads: usize) -> Result<String, Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;

    let (send_out, recv_out) = std::sync::mpsc::channel();

    let threads = (0..num_threads)
        .map(|_| {
            let send_out = send_out.clone();
            let (send_line, recv_line) = std::sync::mpsc::channel::<String>();

            std::thread::spawn(move || {
                let mut station_map = HashMap::<String, WeatherStation>::new();

                for line in recv_line.iter() {
                    if line == SIGNAL_END {
                        break;
                    }

                    let s = line.split(';').collect::<Vec<_>>();
                    let name = s[0];
                    let value = s[1].parse::<f64>().unwrap();

                    let station = match station_map.get_mut(name) {
                        Some(station) => station,
                        None => {
                            station_map.insert(name.to_string(), WeatherStation::default());
                            station_map.get_mut(name).unwrap()
                        }
                    };

                    station.count += 1;
                    station.total += value;
                    station.max = station.max.max(value);
                    station.min = station.min.min(value);
                }

                send_out.send(station_map).expect("Error sending line");
            });

            send_line
        })
        .collect::<Vec<_>>();

    let mut num_lines = 0;
    let mut used_threads = HashSet::new();

    contents.split('\n').for_each(|line| {
        if line.is_empty() {
            return;
        }

        let first_char = line.chars().next().unwrap();

        // Deterministic thread selection using the first character of the line
        let thread_idx = (first_char as usize) % num_threads;
        used_threads.insert(thread_idx);
        let thread = &threads[thread_idx];

        thread.send(line.to_string()).expect("Error sending line");

        num_lines += 1;
    });

    for send in threads {
        send.send(SIGNAL_END.to_string())
            .expect("Error sending line");
    }

    let mut array = recv_out
        .iter()
        .take(used_threads.len())
        .flat_map(|m| m.into_iter().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    array.sort_by(|a, b| a.0.cmp(&b.0));

    let out_file_name = path.replace(".txt", ".out");
    let mut file = std::fs::File::create(out_file_name.clone())?;

    for (k, v) in array {
        let mean = v.total / v.count as f64;
        let line = format!("{}:{};{:.1};{}\n", k, v.min, mean, v.max);
        file.write_all(line.as_bytes())?;
    }

    Ok(out_file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data() {
        let files = std::fs::read_dir("test-data")
            .unwrap()
            .filter_map(|f| f.ok())
            .filter(|f| f.path().to_str().unwrap().ends_with(".txt"));

        for file in files {
            let path = file.path();
            let filename = path.to_str().unwrap();

            println!("Testing {}", filename);

            let out = process_file(filename, 1).expect("Error processing file");
            validate(filename, &out);
        }
    }

    /// Validate that the output file is correct the given input file.
    fn validate(input_file: &str, output_file: &str) {
        let input = std::fs::read_to_string(input_file).unwrap();
        let output = std::fs::read_to_string(output_file).unwrap();

        let mut station_map = std::collections::HashMap::<String, WeatherStation>::new();

        for line in input.split('\n') {
            if line.is_empty() {
                continue;
            }

            let s = line.split(';').collect::<Vec<_>>();
            let name = s[0];
            let value = s[1].parse::<f64>().unwrap();

            let station = station_map.entry(name.to_string()).or_default();

            station.count += 1;
            station.total += value;
            station.max = station.max.max(value);
            station.min = station.min.min(value);
        }

        let mut array = station_map.iter().collect::<Vec<_>>();
        array.sort_by(|a, b| a.0.cmp(b.0));

        let mut expected = String::new();

        for (k, v) in array {
            let mean = v.total / v.count as f64;
            let line = format!("{}:{};{:.1};{}\n", k, v.min, mean, v.max);
            expected.push_str(&line);
        }

        assert_eq!(output, expected);
    }
}
