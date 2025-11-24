use std::collections::BTreeMap;
use std::env;
use std::fs;

// Helper to determine the default file extension representation for files 
// without an extension (like 'start' in the example).
const NO_EXTENSION_PLACEHOLDER: &str = ":";

/// Lists files in the current directory and groups them by their file extension.
/// It ignores directories and hidden files (starting with '.').
fn run_file_sorter() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get the current working directory path.
    let current_dir = env::current_dir()?;

    // Print the directory being scanned for context.
    println!("Scanning directory: {}\n", current_dir.display());

    // Use a BTreeMap to store results. This map automatically keeps the keys 
    // (file extensions) sorted alphabetically, ensuring the final output 
    // is organized as requested.
    let mut files_by_extension: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // 2. Iterate over the entries in the current directory.
    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();

        // 3. Process only regular files.
        if path.is_file() {
            // Extract the filename as a string.
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                
                // Skip files that start with '.' (hidden files) for cleaner output.
                if filename.starts_with('.') {
                    continue; 
                }

                // 4. Determine the file extension.
                let extension = match path.extension().and_then(|ext| ext.to_str()) {
                    // If an extension exists, convert it to lowercase for grouping.
                    Some(ext) => ext.to_lowercase(), 
                    // If no extension, use the defined placeholder (':').
                    None => NO_EXTENSION_PLACEHOLDER.to_string(), 
                };

                // Get the file name (e.g., "document.pdf" -> "document.pdf")
                let filename_only = filename.to_string();
                
                // 5. Insert the filename into the correct extension group.
                // .entry(key).or_default() gets the Vec<String> for the extension
                // or creates a new one if it doesn't exist.
                files_by_extension
                    .entry(extension)
                    .or_default()
                    .push(filename_only);
            }
        }
    }

    // 6. Print the results.
    for (extension, filenames) in files_by_extension {
        // Since BTreeMap iterates in sorted key order (by extension), we only 
        // need to sort the filenames within each group.
        let mut sorted_filenames = filenames;
        sorted_filenames.sort_unstable(); // Use unstable sort for efficiency

        // Print the extension header (e.g., "pdf:").
        println!("{extension}:");

        // Print the list of files.
        for filename in sorted_filenames {
            println!("- {filename}");
        }
        println!(); // Add a blank line for clean separation between groups.
    }

    Ok(())
}

fn main() {
    match run_file_sorter() {
        Ok(()) => {},
        Err(e) => {
            // Print errors to stderr and exit with a non-zero status code.
            eprintln!("An error occurred: {e}");
            std::process::exit(1);
        }
    }
}
