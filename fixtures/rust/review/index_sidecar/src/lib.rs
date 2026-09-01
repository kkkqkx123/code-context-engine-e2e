/// Demonstrates indexed sidecar data for a function with branching and raw behavior facts.
pub fn process_values(values: Option<Vec<i32>>) -> Result<usize, std::io::Error> {
    if let Some(items) = values {
        let mut buffer = Vec::new();
        for item in items {
            if *item < 0 {
                continue;
            }
            buffer.push(*item);
        }

        let borrowed = &buffer;
        let _ = borrowed.len();
        let _ = std::result::Result::<usize, std::io::Error>::Ok(buffer.len())?;
        let _ = 1 << 2;

        let outcome = loop {
            break match buffer.len() {
                0 => Ok(buffer.len()),
                _ => Ok(buffer.len()),
            };
        };

        outcome
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "missing values",
        ));
    }
}
