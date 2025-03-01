use neo::neo_error::NeoError;
use neo::neo_types::{Byte, Bytes};

fn main() -> Result<(), NeoError> {
    println!("Hello, Neo!");
    let bytes: Bytes = vec![1, 2, 3, 4];
    println!("Bytes: {:?}", bytes);
    Ok(())
}
