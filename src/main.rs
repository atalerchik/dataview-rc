use glob::glob;
use regex::Regex;
use std::{collections::HashMap, fs, io, env, path::PathBuf};
use dotenv::dotenv;

fn main() -> io::Result<()> {
    dotenv().ok();
    let vault_path = env::var("VAULT_PATH").unwrap();
    let all_files_path = format!("{}/**/*.md", vault_path);
    let exclude_files_paths = vec![
        format!("{}/home/daily/List - Tasks.md", vault_path),
        format!("{}/home/dates/*.md", vault_path),
        format!("{}/home/wiki/*.md", vault_path),
        format!("{}/meta/*.md", vault_path),
        ];

    // Regex for get all wiki-links in note
    let note_link_re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    // Regex for get dataview command in daily note
    let aggregator_re = Regex::new(r"```dataview\s+list from \[\[\]\]\s+```").unwrap();

    // Daily note is a key for vector of notes with wiki-links in this daily note
    let mut aggregator_links: HashMap<PathBuf, Vec<String>> = HashMap::new();

    // Identify aggregator notes and initialize their links
    for note_path in glob(&all_files_path).expect("Failed to read glob pattern") {
        println!("Note: {}", &note_path.as_ref().unwrap().display());
        match &note_path {
            Ok(path) => {
                let note_content = fs::read_to_string(path)?;
                if exclude_files_paths.contains(&path.display().to_string()) {
                    continue;
                }
                // If it's an aggregator note, add it to the map
                if aggregator_re.is_match(&note_content) {
                    aggregator_links.insert(path.clone(), Vec::new());
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    // Gather links for each aggregator note
    for entry in glob(&all_files_path).expect("Failed to read glob pattern") {
        match &entry {
            Ok(path) => {
                let content = fs::read_to_string(path)?;
                let file_name = path.file_stem().unwrap().to_str().unwrap();

                // Check if this file links to any aggregator note
                for (aggregator_path, links) in aggregator_links.iter_mut() {
                    let aggregator_name = aggregator_path.file_stem().unwrap().to_str().unwrap();
                    if content.contains(&format!("[[{}]]", aggregator_name)) {
                        links.push(format!("[[{}]]", file_name));
                    }
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    // now pretty print HashMap
    for (path, links) in &aggregator_links {
        println!("{}: {:?}", path.display(), links);
    }

    // Update each aggregator note
    for (path, links) in aggregator_links {
        let mut content = fs::read_to_string(&path)?;

        let new_dataview_block = format!(
            "{}",
            links
                .iter()
                .map(|note| format!("- {}\n", note))
                .collect::<Vec<_>>()
                .join("")
        );

        content = aggregator_re.replace(&content, &new_dataview_block).to_string();
        fs::write(path, content)?;
    }

    Ok(())
}
