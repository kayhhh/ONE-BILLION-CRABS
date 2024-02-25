use std::{collections::HashMap, error::Error, io::Write};

#[derive(Debug)]
struct WeatherStation {
    count: usize,
    max: f64,
    mean: f64,
    min: f64,
}

impl Default for WeatherStation {
    fn default() -> Self {
        WeatherStation {
            count: 0,
            max: -std::f64::INFINITY,
            mean: 0.0,
            min: std::f64::INFINITY,
        }
    }
}

pub fn process_file(path: &str) -> Result<(), Box<dyn Error>> {
    let contents = std::fs::read_to_string(path)?;
    let mut map: HashMap<String, WeatherStation> = HashMap::new();

    contents.split('\n').for_each(|line| {
        if line.is_empty() {
            return;
        }

        let s = line.split(';').collect::<Vec<_>>();

        let name = s[0];
        let value = s[1].parse::<f64>().unwrap();

        let station = match map.get_mut(name) {
            Some(station) => station,
            None => {
                map.insert(name.to_string(), WeatherStation::default());
                map.get_mut(name).unwrap()
            }
        };

        station.count += 1;
        station.mean = (station.mean * (station.count - 1) as f64 + value) / station.count as f64;
        station.max = station.max.max(value);
        station.min = station.min.min(value);
    });

    // Sort by name
    let mut array = map.into_iter().collect::<Vec<_>>();
    array.sort_by(|a, b| a.0.cmp(&b.0));

    let out_file_name = path.replace(".txt", ".out");
    println!("Writing output to {}", out_file_name);
    let mut file = std::fs::File::create(out_file_name)?;

    for (k, v) in array {
        let line = format!("{}:{};{:.1};{}\n", k, v.min, v.mean, v.max);
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
