use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse the argument
    let path = std::env::args().take(2).last().unwrap();

    println!("opening {path}");

    let raw_file = File::open(path)?;

    // Parsing time
    let start = std::time::Instant::now();
    let class = classfile::read_from(raw_file)?;
    let end = std::time::Instant::now().duration_since(start);

    println!("Read class: {class:?}");
    println!("Duration: {end:?}");

    Ok(())
}
