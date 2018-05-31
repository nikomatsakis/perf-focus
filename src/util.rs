pub fn percent(num: usize, denom: usize) -> u32 {
    let num: f64 = num as f64;
    let denom: f64 = denom as f64;
    let percent: f64 = num * 100.0 / denom;
    percent as u32
}

pub fn seconds(samples: usize, frequency: usize) -> f64 {
    (samples as f64) / (frequency as f64)
}

pub fn seconds_str(samples: usize, frequency: usize) -> String {
    format!("{:.2}s", seconds(samples, frequency))
}
