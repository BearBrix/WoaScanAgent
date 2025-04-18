use std::collections::HashSet;
use std::env;
use std::error::Error;

use reqwest::blocking::Client;
use serde::Serialize;
use webbrowser;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize)]
struct AppInfo {
    name: String,
    publisher: String,
}

fn perform_scan() -> Vec<AppInfo> {
    let hives = [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    let mut seen = HashSet::new();
    let mut apps = Vec::new();

    for (hive, path) in &hives {
        let key = RegKey::predef(*hive).open_subkey(path);
        if let Ok(key) = key {
            for subkey_name in key.enum_keys().flatten() {
                if let Ok(subkey) = key.open_subkey(&subkey_name) {
                    let name: Result<String, _> = subkey.get_value("DisplayName");
                    let publisher: Result<String, _> = subkey.get_value("Publisher");

                    if let Ok(name) = name {
                        let publisher = publisher.unwrap_or_else(|_| "Unknown Publisher".to_string());

                        if seen.insert(name.clone()) {
                            apps.push(AppInfo { name, publisher });
                        }
                    }
                }
            }
        }
    }

    apps
}

fn main() -> Result<(), Box<dyn Error>> {
    // pass http://localhost:3000 as argument for local development!
    let base_url = env::args().nth(1).unwrap_or_else(|| "https://woascan.app".to_string());
    let register_url = format!("{}/api/register-scan", base_url);

    println!("Scanning system...");
    let scan_results = perform_scan();

    println!("Registering scan with server...");
    let client = Client::new();
    let response = client.post(&register_url).json(&scan_results).send()?;

    if response.status().is_success() {
        let scan_url = response.text()?.trim_matches('"').to_string();

        println!("Server returned URL: {}", scan_url);

        if let Err(e) = webbrowser::open(&scan_url) {
            eprintln!("Failed to open URL in browser: {}", e);
        }
    } else {
        println!("❌ Server returned an error: {}", response.status());
    }

    Ok(())
}
