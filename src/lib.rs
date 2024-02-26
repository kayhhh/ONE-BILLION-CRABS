use std::{collections::BTreeMap, error::Error, io::Write};

use tokio::io::AsyncReadExt;

#[derive(Clone, Debug)]
struct WeatherStation {
    count: u64,
    max: i64,
    min: i64,
    sum: i64,
}

impl Default for WeatherStation {
    fn default() -> Self {
        WeatherStation {
            count: 0,
            max: i64::MIN,
            min: i64::MAX,
            sum: 0,
        }
    }
}

// Size of each chunk we read from the file, in bytes.
const CHUNK_SIZE: usize = 64 * 1024 * 1024;

/// Process the given file, returns the name of the output file.
pub async fn process_file(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = tokio::fs::File::open(path).await?;

    let mut handles = Vec::new();
    let mut extra = Vec::new();

    loop {
        // Read next chunk from file.
        let chunk_size = CHUNK_SIZE - extra.len();
        let mut buf = vec![0; chunk_size];

        let last_loop = match file.read_exact(&mut buf).await {
            Ok(_) => false,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => true,
            Err(e) => return Err(e.into()),
        };

        // Add the extra bytes to start of the buffer.
        let mut new_buf = extra.clone();
        new_buf.extend_from_slice(&buf);

        // Find the last newline in the buffer.
        let mut last_newline = new_buf.len();

        for (i, &c) in new_buf.iter().enumerate().rev() {
            if c == b'\n' {
                last_newline = i;
                break;
            }
        }

        // Split the buffer at the last newline.
        let (lines, new_extra) = new_buf.split_at(last_newline);
        extra = new_extra.to_vec();

        let lines = lines.to_vec();

        // Process the lines.
        let handle = tokio::spawn(async move {
            let mut map = BTreeMap::<String, WeatherStation>::new();

            for line in lines.split(|&c| c == b'\n') {
                if line.is_empty() {
                    continue;
                }

                let line = std::str::from_utf8(line).unwrap();
                let s = line.split(';').collect::<Vec<_>>();
                let name = s[0];
                let value = s[1].parse::<f64>().unwrap();
                let value = (value * 100.0) as i64;

                let station = map.entry(name.to_string()).or_default();

                station.count += 1;
                station.sum += value;
                station.max = station.max.max(value);
                station.min = station.min.min(value);
            }

            map.into_iter().collect::<Vec<_>>()
        });

        handles.push(handle);

        if last_loop {
            break;
        }
    }

    let mut map = BTreeMap::<String, WeatherStation>::new();

    for handle in handles {
        let lines = handle.await.unwrap();

        for (k, v) in lines {
            let station = map.entry(k.to_string()).or_default();

            station.count += v.count;
            station.sum += v.sum;
            station.max = station.max.max(v.max);
            station.min = station.min.min(v.min);
        }
    }

    let out_name = path.replace(".txt", ".out");
    let mut out = std::fs::File::create(&out_name)?;

    for (k, v) in map.iter() {
        let mean = v.sum as f64 / v.count as f64 / 100.0;
        let min = v.min as f64 / 100.0;
        let max = v.max as f64 / 100.0;
        let line = format!("{};{:.1};{:.1};{:.1}\n", k, min, mean, max);
        out.write_all(line.as_bytes())?;
    }

    Ok(out_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct WeatherStation {
        count: u64,
        max: f64,
        min: f64,
        sum: f64,
    }

    impl Default for WeatherStation {
        fn default() -> Self {
            WeatherStation {
                count: 0,
                max: -std::f64::INFINITY,
                min: std::f64::INFINITY,
                sum: 0.0,
            }
        }
    }

    #[tokio::test]
    async fn test_data() {
        let files = std::fs::read_dir("test-data")
            .unwrap()
            .filter_map(|f| f.ok())
            .filter(|f| f.path().to_str().unwrap().ends_with(".txt"));

        for file in files {
            let path = file.path();
            let filename = path.to_str().unwrap();

            println!("Testing {}", filename);

            let out = process_file(filename).await.expect("Error processing file");
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
            station.sum += value;
            station.max = station.max.max(value);
            station.min = station.min.min(value);
        }

        let mut array = station_map.iter().collect::<Vec<_>>();
        array.sort_by(|a, b| a.0.cmp(b.0));

        let mut expected = String::new();

        for (k, v) in array {
            let mean = v.sum / v.count as f64;
            let line = format!("{};{:.1};{:.1};{:.1}\n", k, v.min, mean, v.max);
            expected.push_str(&line);
        }

        assert_eq!(output, expected);
    }
}
