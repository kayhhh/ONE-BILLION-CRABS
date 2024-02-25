use std::{error::Error, io::Write};

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

const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";

pub fn process_file(path: &str) -> Result<(), Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;

    let (send_out, recv_out) = std::sync::mpsc::channel();

    let num_threads = std::thread::available_parallelism().unwrap().get();

    let threads = (0..num_threads)
        .map(|_| {
            let (send_line, recv_line) = std::sync::mpsc::channel::<String>();

            let send_out = send_out.clone();

            std::thread::spawn(move || {
                let mut station_map = std::collections::HashMap::<String, WeatherStation>::new();

                for line in recv_line.iter() {
                    let line = line.to_string();
                    let s = line.split(';').collect::<Vec<_>>();
                    let name = s[0];
                    let value = s[1].parse::<f64>().unwrap();

                    let station = station_map.entry(name.to_string()).or_default();

                    station.count += 1;
                    station.total += value;
                    station.max = station.max.max(value);
                    station.min = station.min.min(value);

                    send_out
                        .send((name.to_string(), station.clone()))
                        .expect("Error sending line");
                }
            });

            send_line
        })
        .collect::<Vec<_>>();

    let mut num_lines = 0;

    contents.split('\n').for_each(|line| {
        if line.is_empty() {
            return;
        }

        let thread_idx = ALPHABET
            .chars()
            .position(|c| c == line.chars().next().unwrap())
            .unwrap()
            % num_threads;

        let thread = &threads[thread_idx];

        thread.send(line.to_string()).expect("Error sending line");

        num_lines += 1;
    });

    for thread in threads {
        drop(thread);
    }

    let mut array = recv_out.iter().take(num_lines).collect::<Vec<_>>();
    array.sort_by(|a, b| a.0.cmp(&b.0));

    let out_file_name = path.replace(".txt", ".out");
    println!("Writing output to {}", out_file_name);
    let mut file = std::fs::File::create(out_file_name)?;

    for (k, v) in array {
        let mean = v.total / v.count as f64;
        let line = format!("{}:{};{:.1};{}\n", k, v.min, mean, v.max);
        file.write_all(line.as_bytes())?;
    }

    Ok(())
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
            process_file(filename).expect("Error processing file");
        }
    }
}
