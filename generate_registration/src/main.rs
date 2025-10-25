use regex::Regex;
use std::fs;
use std::path::Path;

fn main() {
    let mut registrations = Vec::new();

    // Walk through all sort files (look in parent directory)
    let sorts_dir = Path::new("../src/sorts");
    if let Ok(entries) = fs::read_dir(sorts_dir) {
        for entry in entries.flatten() {
            if let Ok(entries) = fs::read_dir(entry.path()) {
                for sub_entry in entries.flatten() {
                    if let Ok(entries) = fs::read_dir(sub_entry.path()) {
                        for file_entry in entries.flatten() {
                            if file_entry
                                .path()
                                .extension()
                                .map_or(false, |ext| ext == "rs")
                            {
                                process_file(&file_entry.path(), &mut registrations);
                            }
                        }
                    } else {
                        // Single level directory
                        if sub_entry
                            .path()
                            .extension()
                            .map_or(false, |ext| ext == "rs")
                        {
                            process_file(&sub_entry.path(), &mut registrations);
                        }
                    }
                }
            }
        }
    }

    // Generate the registration code
    println!("// Auto-generated sort registrations");
    println!("// Add this to your main.rs or lib.rs");
    println!();
    println!("use crate::traits::{{init_sort_registry, register_sort}};");
    println!();
    println!("pub fn register_all_sorts() {{");
    println!("    init_sort_registry();");
    println!();

    for (name, big_o, stable, category) in registrations {
        println!(
            "    register_sort(\"{}\", \"{}\", {}, \"{}\");",
            name, big_o, stable, category
        );
    }

    println!("}}");
}

fn process_file(path: &Path, registrations: &mut Vec<(String, String, bool, String)>) {
    if let Ok(content) = fs::read_to_string(path) {
        // Extract category from path
        let category = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Find create_sort! macro calls
        let re = Regex::new(
            r#"create_sort!\s*\(\s*sort\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(true|false)\s*\)"#,
        )
        .unwrap();

        for cap in re.captures_iter(&content) {
            let mut name = cap[1].to_string();
            let big_o = cap[2].to_string();
            let stable = &cap[3] == "true";

            // Create unique names for sorts that have the same name
            let filename = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // If the name is generic like "circle_sort" or "heap sort", make it unique
            if name == "circle_sort" || name == "heap sort" {
                name = format!("{} {}", name, filename.replace("_", " "));
            }

            // Check if this exact combination already exists
            let key = (name.clone(), big_o.clone(), stable, category.clone());
            if !registrations.iter().any(|(n, b, s, c)| n == &name && b == &big_o && s == &stable && c == &category) {
                registrations.push((name, big_o, stable, category.clone()));
            }
        }
    }
}
