use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse the argument
    let path = std::env::args().take(2).last().unwrap();

    println!("opening {path}");

    let raw_file = File::open(path)?;
    let class = classfile::read_from(raw_file)?;

    println!("Read class: {class:?}");

    Ok(())
}
