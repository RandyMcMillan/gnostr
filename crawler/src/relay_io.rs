use crate::processor::BOOTSTRAP_RELAYS;
use log::{debug, trace, warn};
use nostr_sdk::prelude::Url;
use std::collections::HashSet;
use std::fs as sync_fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn preprocess_line(line: &str) -> String {
    let mut trimmed_line = line.trim().to_string();
    if let Some(stripped) = trimmed_line.strip_prefix("- ") {
        trimmed_line = stripped.trim().to_string();
    } else if let Some(stripped) = trimmed_line.strip_prefix('-') {
        trimmed_line = stripped.trim().to_string();
    }
    if let Some(comma_idx) = trimmed_line.find(',') {
        trimmed_line.truncate(comma_idx);
        trimmed_line = trimmed_line.trim().to_string();
    }
    trimmed_line
}

fn normalize_relay_entry(line: &str) -> Option<String> {
    let mut final_line = preprocess_line(line);
    if final_line.is_empty() {
        return None;
    }

    if !final_line.contains("://") {
        let potential_url = format!("wss://{}", final_line);
        if let Ok(url) = Url::parse(&potential_url) {
            debug!("Prepended 'wss://' to form valid URL: {}", url);
            final_line = url.to_string();
        }
    }

    if final_line.starts_with("wss://") || final_line.starts_with("ws://") {
        match Url::parse(&final_line) {
            Ok(url) => Some(url.to_string()),
            Err(_) => {
                trace!("Skipping invalid WEBSOCKET URL: {}", final_line);
                None
            }
        }
    } else {
        None
    }
}

pub(crate) fn parse_relay_entries(content: &str) -> Vec<String> {
    if let Ok(values) = serde_yaml::from_str::<Vec<String>>(content) {
        return values
            .into_iter()
            .filter_map(|line| normalize_relay_entry(&line))
            .collect();
    }

    content
        .lines()
        .filter_map(normalize_relay_entry)
        .collect()
}

pub fn load_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let base_dir = crate::relays::get_config_dir_path();
    let file_path = base_dir.join(
        filename
            .as_ref()
            .file_name()
            .unwrap_or(filename.as_ref().as_os_str()),
    );

    if let Some(parent) = file_path.parent() {
        sync_fs::create_dir_all(parent)?;
    }

    debug!("load_file: start path={}", file_path.display());

    let file_content = sync_fs::read_to_string(&file_path)?;
    let relays = parse_relay_entries(&file_content);
    debug!(
        "load_file: parsed {} relay entries from {}",
        relays.len(),
        file_path.display()
    );
    Ok(relays)
}

pub fn load_shitlist(filename: impl AsRef<Path>) -> io::Result<HashSet<String>> {
    let path = filename.as_ref().to_path_buf();
    debug!("load_shitlist: start path={}", path.display());
    let entries = BufReader::new(sync_fs::File::open(&path)?)
        .lines()
        .collect::<io::Result<HashSet<String>>>()?;
    debug!(
        "load_shitlist: loaded {} entries from {}",
        entries.len(),
        path.display()
    );
    Ok(entries)
}

pub fn load_relays_or_bootstrap() -> Vec<String> {
    debug!("load_relays_or_bootstrap: start");
    match load_file("relays.yaml") {
        Ok(relays) => relays,
        Err(e) => {
            warn!(
                "Failed to load relays.yaml ({}); falling back to bootstrap relays",
                e
            );
            let relays: Vec<String> = BOOTSTRAP_RELAYS.iter().cloned().collect();
            debug!(
                "load_relays_or_bootstrap: using {} bootstrap relays",
                relays.len()
            );
            relays
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_relay_entries;

    #[test]
    fn parses_yaml_relays_without_list_markers() {
        let entries = parse_relay_entries(
            "- wss://relay.example/\n- relay.example\n- wss://relay2.example/\n",
        );

        assert_eq!(
            entries,
            vec![
                "wss://relay.example/".to_string(),
                "wss://relay.example/".to_string(),
                "wss://relay2.example/".to_string(),
            ]
        );
    }

    #[test]
    fn parses_plain_text_relays() {
        let entries = parse_relay_entries("relay.example\nwss://relay2.example/\n");

        assert_eq!(
            entries,
            vec![
                "wss://relay.example/".to_string(),
                "wss://relay2.example/".to_string(),
            ]
        );
    }
}
