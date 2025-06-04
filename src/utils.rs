use std::collections::HashMap;
use std::fs::File as StdFile;
use std::io::{BufRead, BufReader as StdBufReader};

pub fn load_optimal_solutions(file_path: &str) -> Result<HashMap<String, f64>, String> {
    let file = StdFile::open(file_path)
        .map_err(|e| format!("Failed to open solutions file {}: {}", file_path, e))?;
    let reader = StdBufReader::new(file);
    let mut solutions = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| format!("Error reading solution line: {}", e))?;
        let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let name_part = parts[0];
            let clean_name = name_part
                .split_whitespace()
                .next()
                .unwrap_or(name_part)
                .to_lowercase();

            let value_str_full = parts[1];
            let value_str_numeric = value_str_full
                .split_whitespace()
                .next()
                .unwrap_or(value_str_full);

            let value = value_str_numeric.parse::<f64>().map_err(|e| {
                format!(
                    "Invalid solution value for {} (from '{}'): {}",
                    clean_name, value_str_full, e
                )
            })?;
            solutions.insert(clean_name, value);
        }
    }
    Ok(solutions)
}

pub fn evaluate_solution(
    problem_name: &str,
    found_length: f64,
    optimal_solutions: &HashMap<String, f64>,
) -> (Option<f64>, Option<f64>) {
    let key_name = problem_name.to_lowercase();
    if let Some(optimal_length) = optimal_solutions.get(&key_name) {
        let percentage_diff = if *optimal_length == 0.0 {
            if found_length == 0.0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            ((found_length - optimal_length) / optimal_length) * 100.0
        };
        (Some(*optimal_length), Some(percentage_diff))
    } else {
        (None, None)
    }
}
