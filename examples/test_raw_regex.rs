use std::time::Instant;

fn main() {
    // Very simple regex test
    use regex::Regex;
    use once_cell::sync::Lazy;
    
    static TEST_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#).unwrap()
    });
    
    let test_text = r#"rule "CalculateShortage" salience 120 no-loop {
        when
            required_qty > 0
        then
            Log("Calculating shortage...");
            shortage = required_qty - available_qty;
            Log("Shortage calculated");
    }"#;
    
    // Warm up
    let _ = TEST_REGEX.captures(test_text);
    
    // Measure
    let iterations = 100000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = TEST_REGEX.captures(test_text);
    }
    let elapsed = start.elapsed();
    
    let micros = elapsed.as_micros() as f64 / iterations as f64;
    println!("Regex captures: {:.2} µs per call", micros);
    println!("Total: {:?}", elapsed);
    println!("Throughput: {:.0} per sec", iterations as f64 / elapsed.as_secs_f64());
}
