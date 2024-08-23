use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

fn sha1_hash(password: &str) -> String {
    let mut sha_pass = Sha1::new();
    sha_pass.update(password);
    let result = sha_pass.finalize();
    let target_hash = result
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<String>();
    target_hash
}

fn binary_search(file_path: &str, target_hash: &str) -> io::Result<Option<String>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut low = 0;
    let mut high = reader.seek(SeekFrom::End(0))?;
    let mut iterations = 0;

    while low < high {
        iterations += 1;
        let mid = (low + high) / 2;

        // Move to the middle of the file
        reader.seek(SeekFrom::Start(mid))?;

        // Read until the end of the current line to ensure we are at the start of a new line
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;

        // If we are not at the start of the file, read the previous line to ensure we are at the start of a line
        if mid != 0 {
            buffer.clear();
            reader.seek(SeekFrom::Current(-(buffer.len() as i64 + 1)))?;
            reader.read_line(&mut buffer)?;
        }

        // Read the next full line to get the actual mid line
        buffer.clear();
        reader.read_line(&mut buffer)?;
        let mid_line = buffer.trim_end();
        let mid_hash = mid_line.split(':').next().unwrap_or("");

        println!(
            "Iteration {}: Comparing target hash {} with {}",
            iterations, target_hash, mid_hash
        );

        match mid_hash.cmp(target_hash) {
            std::cmp::Ordering::Equal => {
                println!("Total iterations: {}", iterations);
                return Ok(Some(mid_line.to_string()));
            }
            std::cmp::Ordering::Less => low = reader.stream_position()?,
            std::cmp::Ordering::Greater => {
                // Adjust high to the start of the current line
                high = mid;
                while high > 0 {
                    high -= 1;
                    reader.seek(SeekFrom::Start(high))?;
                    let mut temp_buffer = [0; 1];
                    reader.read_exact(&mut temp_buffer).unwrap();
                    if temp_buffer[0] == b'\n' {
                        high += 1;
                        break;
                    }
                }
            }
        }
    }

    println!("Total iterations: {}", iterations);
    Ok(None)
}

fn is_valid_sha1_hash(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_digit(16))
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <password_or_hash>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    let input = &args[2];
    let target_hash = if is_valid_sha1_hash(input) {
        input.to_string()
    } else {
        sha1_hash(input)
    };

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    if reader.lines().next().is_none() {
        eprintln!("The file is empty.");
        std::process::exit(1);
    }

    match binary_search(file_path, &target_hash)? {
        Some(line) => println!("Found: {}", line),
        None => println!("Hash not found."),
    }

    Ok(())
}
