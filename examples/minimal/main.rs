use neo3::{neo_error::NeoError, neo_types::bytes::Bytes};

fn main() -> Result<(), NeoError> {
    println!("Hello, Neo!");
    let bytes: Bytes = vec![1, 2, 3, 4].into();
    println!("Bytes: {:?}", bytes);
    Ok(())
}
