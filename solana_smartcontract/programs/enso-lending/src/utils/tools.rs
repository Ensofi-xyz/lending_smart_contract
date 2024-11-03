pub fn duration_to_year(duration: u64) -> f64 {
    (duration as f64) / ((24 * 60 * 60 * 365) as f64)
}