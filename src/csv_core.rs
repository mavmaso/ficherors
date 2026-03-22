use csv::{Reader, ReaderBuilder};
use regex::Regex;
use std::io::{Cursor, Read};
use std::fs::File;
use std::io::{BufReader, BufRead};
use anyhow::{Result, Context, anyhow};


pub struct CsvData {
    pub placeholders: Vec<String>,
    pub data: Vec<Vec<String>>,
}

pub struct FileData {
    pub valid: bool,
    pub errors: Vec<String>,
    pub error_type: String,
    pub destination_count: usize,
}

/// Opens a file and returns a CSV Reader with the detected delimiter.
pub fn csv_reader(path: &str) -> Result<Reader<File>> {
    let f = File::open(path).with_context(|| format!("failed to open file `{}`", path))?;
    let buf = BufReader::new(f);

    let separator = match buf.lines().next() {
        Some(Ok(first_line)) => get_separator(&first_line),
        Some(Err(e)) => return Err(anyhow!(e)).context("failed to read first line"),
        None => return Err(anyhow!("separator_error")),
    };

    let reader = ReaderBuilder::new()
        .delimiter(separator)
        .from_path(path)
        .with_context(|| format!("failed to open csv at `{}`", path))?;

    Ok(reader)
}

pub fn path_to_csv_data(path: &str) -> Result<CsvData> {
    let reader = csv_reader(path)?;
    create_csv_data(reader)
}

pub fn create_csv_data<T: Read>(mut reader: Reader<T>) -> Result<CsvData> {
    let placeholders = reader
        .headers()
        .context("failed to read CSV headers")?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    check_header(&placeholders)?;

    let mut data: Vec<Vec<String>> = vec![];
    for row in reader.records() {
        let r = row.context("row_format")?;
        let line = r.into_iter().map(|f| f.to_string()).collect();
        data.push(line);
    }

    let csv_data = CsvData { placeholders, data };

    Ok(csv_data)
}

fn check_header(header: &Vec<String>) -> Result<()> {
    if header.contains(&String::from("")) {
        return Err(anyhow!("empty_header"));
    };

    let mut buffer = Vec::new();

    for elem in header {
        if buffer.contains(&elem) {
            return Err(anyhow!("duplicate_headers"));
        } else {
            buffer.push(elem);
        }
    }

    Ok(())
}

// fn verify_telefones removed – logic moved into verify_content

pub fn get_separator(line: &str) -> u8 {
    match true {
        _ if line.contains(';') => b';',
        _ if line.contains('|') => b'|',
        _ if line.contains('\t') => b'\t',
        _ if line.contains(',') => b',',
        _ if line.contains(' ') => b' ',
        _ => b',',
    }
}

/// Wraps a text string into a CSV Reader using auto-detected delimiter.
pub fn text_to_reader(text: &str) -> Result<Reader<Cursor<Vec<u8>>>> {
    let first_line = text.lines().next().unwrap_or("");
    let separator = get_separator(first_line);
    let cursor = Cursor::new(text.as_bytes().to_vec());
    let reader = ReaderBuilder::new()
        .delimiter(separator)
        .from_reader(cursor);
    Ok(reader)
}

/// Validates CSV content and returns a FileData summary.
pub fn verify_content<T: Read>(mut reader: Reader<T>) -> Result<FileData> {
    let headers: Vec<String> = reader
        .headers()
        .context("failed to read CSV headers")?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if headers.contains(&String::from("")) {
        return Ok(FileData {
            valid: false,
            errors: vec!["Invalid header\n".to_string()],
            error_type: "critical error".to_string(),
            destination_count: 0,
        });
    }

    let mut rows: Vec<Vec<String>> = vec![];
    for row in reader.records() {
        let r = row.context("row_format")?;
        rows.push(r.iter().map(|f| f.to_string()).collect());
    }

    if rows.is_empty() {
        return Ok(FileData {
            valid: false,
            errors: vec!["No rows found\n".to_string()],
            error_type: String::new(),
            destination_count: 0,
        });
    }

    let phone_regex = Regex::new(r"^\s*\d+\s*$").expect("static regex is valid");
    let mut errors: Vec<String> = vec![];
    let mut destination_count = 0usize;

    for (index, row) in rows.iter().enumerate() {
        let phone = row.first().map(|s| s.as_str()).unwrap_or("");
        if phone_regex.is_match(phone) {
            destination_count += 1;
        } else {
            errors.push(format!(
                "Error on line {}: {} invalid telephone\n",
                index + 2,
                phone
            ));
        }
    }

    Ok(FileData {
        valid: errors.is_empty(),
        errors,
        error_type: String::new(),
        destination_count,
    })
}

/// Detects whether a file uses CR (Windows \r\n) or LF (Unix \n) line endings.
pub fn detect_line_terminator(path: &str) -> Result<String> {
    let content = std::fs::read(path)
        .with_context(|| format!("failed to open file `{}`", path))?;

    if content.contains(&b'\r') {
        Ok("CR".to_string())
    } else {
        Ok("LF".to_string())
    }
}

/// Strips carriage-return characters from CSV headers (handles CR/CRLF files).
pub fn clean_headers(headers: Vec<String>) -> Vec<String> {
    headers.into_iter().map(|h| h.replace('\r', "")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_separator() {
        assert_eq!(get_separator("a;b;c"), b';');
        assert_eq!(get_separator("a|b|c"), b'|');
        assert_eq!(get_separator("a\tb\tc"), b'\t');
        assert_eq!(get_separator("a,b,c"), b',');
        assert_eq!(get_separator("a b c"), b' ');
        assert_eq!(get_separator("abc"), b',');
    }

    #[test]
    fn test_clean_headers() {
        let headers = vec!["name\r".to_string(), "phone".to_string()];
        assert_eq!(clean_headers(headers), vec!["name", "phone"]);
    }

    #[test]
    fn test_verify_content_valid() {
        let csv = "destination,organization\n5511933336666,Movile\n";
        let reader = text_to_reader(csv).unwrap();
        let result = verify_content(reader).unwrap();
        assert!(result.valid);
        assert_eq!(result.destination_count, 1);
    }

    #[test]
    fn test_verify_content_empty_header() {
        let csv = "destination,,organization\n5511933336666,x,y\n";
        let reader = text_to_reader(csv).unwrap();
        let result = verify_content(reader).unwrap();
        assert!(!result.valid);
        assert_eq!(result.error_type, "critical error");
        assert_eq!(result.errors, vec!["Invalid header\n"]);
    }

    #[test]
    fn test_verify_content_no_rows() {
        let csv = "destination,organization\n";
        let reader = text_to_reader(csv).unwrap();
        let result = verify_content(reader).unwrap();
        assert!(!result.valid);
        assert_eq!(result.errors, vec!["No rows found\n"]);
    }

    #[test]
    fn test_detect_line_terminator_lf() {
        let result = detect_line_terminator("tests/test_files/valid_1.csv").unwrap();
        assert_eq!(result, "LF");
    }

    #[test]
    fn test_detect_line_terminator_cr() {
        let result = detect_line_terminator("tests/test_files/comma_cr.csv").unwrap();
        assert_eq!(result, "CR");
    }

    #[test]
    fn test_detect_line_terminator_not_found() {
        let result = detect_line_terminator("tests/test_files/nonexistent.csv");
        assert!(result.is_err());
    }
}
