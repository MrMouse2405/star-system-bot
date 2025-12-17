use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;

fn main() {
    model::download_model_files();
    tauri_build::build()
}

mod model {
    use super::*;
    use indicatif::{ProgressBar, ProgressStyle};
    use reqwest::blocking::Client;
    use std::time::Duration;

    pub const MODEL_OUTPUT_DIR: &str = "model";
    const SPM_URL: &str = "https://huggingface.co/facebook/m2m100_418M/resolve/main/sentencepiece.bpe.model?download=true";
    const M2M100_URL: &str =
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/rust_model.ot?download=true";
    const VOCAB_URL: &str =
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/vocab.json?download=true";
    const CONFIG_URL: &str =
        "https://huggingface.co/facebook/m2m100_418M/resolve/main/config.json?download=true";
    const QWEN3_URL: &str =
        "https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q5_K_M.gguf?download=true";

    struct ModelFile<'a> {
        filename: &'a str,
        url: &'a str,
    }

    pub fn download_model_files() {
        let files = vec![
            ModelFile {
                filename: "sentencepiece.bpe.model",
                url: SPM_URL,
            },
            ModelFile {
                filename: "rust_model.ot",
                url: M2M100_URL,
            },
            ModelFile {
                filename: "vocab.json",
                url: VOCAB_URL,
            },
            ModelFile {
                filename: "config.json",
                url: CONFIG_URL,
            },
            ModelFile {
                filename: "Qwen3-8B-Q5_K_M.gguf",
                url: QWEN3_URL,
            },
        ];

        // 1. Create directory if it doesn't exist
        let output_dir = Path::new(MODEL_OUTPUT_DIR);
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).expect("Failed to create model directory");
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(1800)) // 30 mins timeout for large files
            .build()
            .unwrap();

        // 2. Loop through files
        for file in files {
            let dest_path = output_dir.join(file.filename);

            if dest_path.exists() {
                // Determine style to look like Cargo's "    Finished ..."
                print_cargo_style("Skipping", &format!("{} (already exists)", file.filename));
                continue;
            }

            download_file(&client, file.url, &dest_path, file.filename);
        }
    }

    fn download_file(client: &Client, url: &str, path: &Path, filename: &str) {
        print_cargo_style("Downloading", filename);

        let mut response = client.get(url).send().expect("Failed to send request");

        // Get content length for progress bar
        let total_size = response
            .content_length()
            .ok_or("Failed to get content length")
            .unwrap_or(0);

        // Setup the Progress Bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        // Create the file
        let mut dest_file = fs::File::create(path).expect("Failed to create file");

        // Stream copy with progress
        let mut buffer = [0; 8192];
        let mut downloaded: u64 = 0;

        loop {
            let bytes_read = response
                .read(&mut buffer)
                .expect("Failed to read from stream");
            if bytes_read == 0 {
                break;
            }
            dest_file
                .write_all(&buffer[..bytes_read])
                .expect("Failed to write to file");

            downloaded += bytes_read as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Done");
    }

    // Helper to print formatted messages like "       Downloading ..."
    fn print_cargo_style(status: &str, message: &str) {
        // Cargo uses 12-character right-aligned tags
        eprintln!("{:>12} {}", status, message);
    }
}
