use anyhow::Result;
use neo3::prelude::*;
use neo3::fs::{NeoFSClient, NeoFSConfig, Container, Object};
use std::path::Path;

const PART_SIZE: usize = 5 * 1024 * 1024; // 5 MB part size

#[tokio::main]
async fn main() -> Result<()> {
    // Load wallet
    let wallet = neo3::wallets::Wallet::load_from_file("wallet.json")?;
    let account = wallet.default_account().unwrap().decrypt("password")?;
    
    // Create NeoFS client with account
    let client = NeoFSClient::default().with_account(account);
    
    // Create a container for our files
    let mut container = Container::new();
    container.set_name("Large Files Container");
    container.attributes.add("Type", "LargeFiles");
    
    // Note: These operations will return NotImplemented error until the gRPC client implementation is complete
    match client.create_container(&container).await {
        Ok(container_id) => {
            println!("Container created: {}", container_id);
            
            // Local large file to upload
            let file_path = Path::new("large_file.dat");
            
            if file_path.exists() {
                println!("Uploading large file using multipart upload...");
                
                // Get file metadata
                let metadata = std::fs::metadata(file_path)?;
                println!("File size: {} bytes", metadata.len());
                
                // Create object with basic metadata (no payload yet)
                let mut object = Object::new(container_id.clone(), client.get_owner_id()?);
                object.set_filename(file_path.file_name().unwrap().to_string_lossy().into_owned());
                object.set_content_type("application/octet-stream");
                
                // Initiate multipart upload
                match client.initiate_multipart_upload(&container_id, &object).await {
                    Ok(upload) => {
                        println!("Multipart upload initiated: {}", upload.upload_id);
                        
                        // Open file for reading
                        let file = std::fs::File::open(file_path)?;
                        let mut reader = std::io::BufReader::new(file);
                        let mut buffer = vec![0u8; PART_SIZE];
                        let mut part_number = 1;
                        let mut parts = Vec::new();
                        
                        // Read and upload parts
                        loop {
                            let bytes_read = std::io::Read::read(&mut reader, &mut buffer)?;
                            if bytes_read == 0 {
                                break; // End of file
                            }
                            
                            println!("Uploading part {} ({} bytes)...", part_number, bytes_read);
                            
                            // Use the actual bytes read
                            let part_data = buffer[0..bytes_read].to_vec();
                            
                            // Upload the part
                            match client.upload_part(&upload, part_number, part_data).await {
                                Ok(part) => {
                                    println!("Part {} uploaded: {}", part_number, part.etag);
                                    parts.push(part);
                                    part_number += 1;
                                },
                                Err(e) => {
                                    println!("Error uploading part {}: {}", part_number, e);
                                    
                                    // Abort the multipart upload
                                    match client.abort_multipart_upload(&upload).await {
                                        Ok(_) => println!("Multipart upload aborted"),
                                        Err(e) => println!("Error aborting multipart upload: {}", e),
                                    }
                                    
                                    return Ok(());
                                }
                            }
                        }
                        
                        // Complete the multipart upload
                        println!("Completing multipart upload with {} parts...", parts.len());
                        match client.complete_multipart_upload(&upload, parts).await {
                            Ok(result) => {
                                println!("Multipart upload completed successfully");
                                println!("Object ID: {}", result.object_id);
                            },
                            Err(e) => println!("Error completing multipart upload: {}", e),
                        }
                    },
                    Err(e) => println!("Error initiating multipart upload: {}", e),
                }
            } else {
                println!("Large file not found: {}", file_path.display());
            }
        },
        Err(e) => println!("Error creating container: {}", e),
    }
    
    Ok(())
}
