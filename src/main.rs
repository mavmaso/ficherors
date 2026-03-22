use std::collections::HashMap;
use csv::WriterBuilder;
use anyhow::{Result, anyhow};
use unidecode::unidecode;

mod countries;
mod csv_core;
mod excel_core;
mod functions;
mod phone;

type Maps = HashMap<String, HashMap<String, String>>;

pub struct Metadata {
    pub country_code: String,
    pub has_accent: bool,
    pub new_path: String,
}

fn main() {
    println!("Running ...");

    let metadata = Metadata {
        country_code: "BR".to_string(),
        has_accent: false,
        new_path: "destinations.csv".to_string(),
    };

    let res = process_csv("20mb.txt", HashMap::new(), metadata).expect("process_csv failed");

    println!("Finished: {}", res);
}

/// Reads a CSV/text file and returns its content as a semicolon-separated string.
pub fn csv_reader(path: &str) -> Result<String> {
    let csv_data = csv_core::path_to_csv_data(path)?;
    let headers = csv_data.placeholders.join(";") + "\n";
    let mut content = headers;

    for row in csv_data.data {
        let line = row.join(";") + "\n";
        content.push_str(&line);
    }

    Ok(content)
}

/// Reads an Excel (.xlsx) file and returns its content as a semicolon-separated string.
pub fn excel_reader(path: &str) -> Result<String> {
    excel_core::path_to_text(path)
}

/// Validates a CSV/text file and returns a summary (destination count, errors, etc.).
pub fn csv_verify(path: &str) -> Result<csv_core::FileData> {
    let reader = csv_core::csv_reader(path)?;
    csv_core::verify_content(reader)
}

/// Validates an Excel file and returns a summary.
pub fn excel_verify(path: &str) -> Result<csv_core::FileData> {
    let text = excel_core::path_to_text(path)?;
    let reader = csv_core::text_to_reader(&text)?;
    csv_core::verify_content(reader)
}

/// Detects whether a file uses CR (Windows) or LF (Unix) line endings.
pub fn detect_terminator(path: &str) -> Result<String> {
    csv_core::detect_line_terminator(path)
}

/// Processes a CSV/text file: formats phone numbers, applies column transforms, writes output.
pub fn process_csv(path: &str, functions: Maps, metadata: Metadata) -> Result<String> {
    let csv_data = csv_core::path_to_csv_data(path).map_err(|e| anyhow!(e))?;

    let country_code = metadata.country_code.trim();
    let remove_accents = !metadata.has_accent;
    let output_path = metadata.new_path.clone();
    let with_functions = !functions.is_empty();
    let old_headers = csv_data.placeholders.clone();
    let mut leftover_headers = vec![];
    let mut headers = vec!["d3stinati0n".to_string()];

    if with_functions {
        functions.keys().for_each(|k| headers.push(k.to_string()));
        let keys: Vec<String> = functions.keys().map(|k| k.to_string()).collect();

        old_headers
            .iter()
            .filter(|i| !keys.contains(i))
            .for_each(|filtered| leftover_headers.push(filtered.to_string()));

        headers.extend(leftover_headers.clone());
    } else {
        headers.extend(old_headers.clone());
    }

    let mut writer = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(&output_path)
        .map_err(|e| anyhow!(e))?;

    let clean_headers = csv_core::clean_headers(headers);
    writer.write_record(clean_headers).map_err(|e| anyhow!(e))?;

    for line in csv_data.data {
        let phone_number = line[0].clone();
        let mut new_row = vec![phone::format_destination(&phone_number, country_code)];

        if with_functions {
            functions::fill_row(&mut new_row, &functions, &line, &old_headers);

            for (index, value) in old_headers.iter().enumerate() {
                if leftover_headers.contains(value) {
                    new_row.push(line[index].clone());
                }
            }
        } else {
            line.into_iter().for_each(|l| new_row.push(l));
        }

        let final_row: Vec<String> = if remove_accents {
            new_row.into_iter().map(|c| unidecode(&c)).collect()
        } else {
            new_row
        };

        writer.write_record(final_row).map_err(|e| anyhow!(e))?;
    }

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_metadata(path: &str) -> Metadata {
        Metadata {
            country_code: String::new(),
            has_accent: true,
            new_path: path.to_string(),
        }
    }

    #[test]
    fn test_csv_reader_valid_comma() {
        let result = csv_reader("tests/test_files/valid_1.csv");
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("destination;name;organization;nickname\n"));
        assert!(content.contains("5516912345678"));
    }

    #[test]
    fn test_csv_reader_valid_semicolon() {
        let result = csv_reader("tests/test_files/valid_3.csv");
        assert!(result.is_ok());
    }

    #[test]
    fn test_csv_reader_valid_tab() {
        let result = csv_reader("tests/test_files/valid_4.csv");
        assert!(result.is_ok());
    }

    #[test]
    fn test_csv_reader_empty_header() {
        let result = csv_reader("tests/test_files/invalid_empty_column_headers.csv");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("empty_header"));
    }

    #[test]
    fn test_csv_reader_not_found() {
        let result = csv_reader("tests/test_files/non.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_reader_duplicate_headers() {
        let result = csv_reader("tests/test_files/invalid_duplicate_headers.csv");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("duplicate_headers"));
    }

    #[test]
    fn test_csv_verify_valid() {
        let result = csv_verify("tests/test_files/valid_1.csv");
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.destination_count, 2);
    }

    #[test]
    fn test_csv_verify_only_headers() {
        let result = csv_verify("tests/test_files/invalid_only_headers.csv");
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.valid);
        assert_eq!(data.errors, vec!["No rows found\n"]);
    }

    #[test]
    fn test_csv_verify_empty_column() {
        let result = csv_verify("tests/test_files/invalid_empty_column_headers.csv");
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.valid);
        assert_eq!(data.error_type, "critical error");
        assert_eq!(data.errors, vec!["Invalid header\n"]);
    }

    #[test]
    fn test_detect_terminator_lf() {
        let result = detect_terminator("tests/test_files/valid_1.csv");
        assert!(matches!(result, Ok(ref s) if s == "LF"));
    }

    #[test]
    fn test_detect_terminator_cr() {
        let result = detect_terminator("tests/test_files/comma_cr.csv");
        assert!(matches!(result, Ok(ref s) if s == "CR"));
    }

    #[test]
    fn test_detect_terminator_not_found() {
        let result = detect_terminator("tests/test_files/nonexistent.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_csv_with_country_code() {
        let metadata = Metadata {
            country_code: "BR".to_string(),
            has_accent: true,
            new_path: "/tmp/test_process_br.csv".to_string(),
        };
        let result = process_csv("tests/test_files/valid_br_phones.csv", HashMap::new(), metadata);
        assert!(result.is_ok());
        let out = std::fs::read_to_string("/tmp/test_process_br.csv").unwrap();
        let rows: Vec<&str> = out.split('\n').collect();
        let headers: Vec<&str> = rows[0].split(';').collect();
        let first_row: Vec<&str> = rows[1].split(';').collect();
        let idx = headers.iter().position(|&h| h == "d3stinati0n").unwrap();
        assert_eq!(first_row[idx], "5516912345678");
    }

    #[test]
    fn test_process_csv_remove_accents() {
        let metadata = Metadata {
            country_code: String::new(),
            has_accent: false,
            new_path: "/tmp/test_process_accent.csv".to_string(),
        };
        let result = process_csv("tests/test_files/valid_w_accent.txt", HashMap::new(), metadata);
        assert!(result.is_ok());
        let out = std::fs::read_to_string("/tmp/test_process_accent.csv").unwrap();
        assert!(out.contains("Voce"));
        assert!(!out.contains("Você"));
    }

    #[test]
    fn test_process_csv_cr_file() {
        let metadata = Metadata {
            country_code: String::new(),
            has_accent: false,
            new_path: "/tmp/test_comma_cr.csv".to_string(),
        };
        let result = process_csv("tests/test_files/comma_cr.csv", HashMap::new(), metadata);
        assert!(result.is_ok());
        let out = std::fs::read_to_string("/tmp/test_comma_cr.csv").unwrap();
        assert!(out.contains("d3stinati0n;destination;name;organization;nickname"));
        assert!(out.contains("5512997517615"));
    }

    #[test]
    fn test_process_csv_spaces_file() {
        let metadata = make_metadata("/tmp/test_spaces.csv");
        let result = process_csv("tests/test_files/valid_w_spaces.txt", HashMap::new(), metadata);
        assert!(result.is_ok());
        let out = std::fs::read_to_string("/tmp/test_spaces.csv").unwrap();
        assert!(out.contains("5516912345672"));
    }
}

