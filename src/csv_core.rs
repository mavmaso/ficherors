use csv::{Reader, ReaderBuilder, WriterBuilder};
use regex::Regex;
use std::collections::HashMap;
use std::io::Read;
use std::fs::File;

pub struct CsvData {
    pub placeholders: Vec<String>,
    pub data: Vec<Vec<String>>,
}

pub struct FileData {
    valid: bool,
    destination_count: usize,
    placeholders: Vec<String>,
    example_data: HashMap<String, String>,
    errors: Vec<String>,
}

pub fn to_text(data: Vec<Vec<String>>) -> Result<String, String> {
    let mut writer = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_writer(vec![]);

    for record in data {
        writer
            .write_record(record)
            .map_err(|_| Err("write_error"))?;
    }

    writer.flush().map_err(|_| Err("flush_error"))?;

    let csv_bin = writer
        .into_inner()
        .map_err(|_| Err("inner_erorr"))?;
    let csv_string = String::from_utf8(csv_bin).map_err(|_| Err("stringfy_error"))?;

    Ok(csv_string)
}

pub fn path_to_csv_data(path: &str) -> Result<CsvData, String> {
    let file = File::open(path).map_err(|_| Err("file_not_found"))?;

    let reader = BufReader::new(file);

    let separator = match reader.lines().next() {
        Some(Ok(first_line)) => get_separator(&first_line),
        _ => return Err(Err("separator_error")),
    };

    let file = ReaderBuilder::new()
        .delimiter(separator)
        .from_path(path)
        .map_err(|_| Err("csv_not_found"))?;

    let csv_data = create_csv_data(file)?;

    Ok(csv_data)
}

pub fn verify_content(content: &str) -> Result<FileData, String> {
    let mut valid = true;
    let mut errors: Vec<String> = vec![];

    let separator = get_separator(content);

    let mut reader = ReaderBuilder::new()
        .delimiter(separator)
        .from_reader(content.as_bytes());

    let placeholders = reader
        .headers()
        .map_err(|_| Err("header_error"))?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    match check_header(&placeholders) {
        Ok(_) => (),
        Err(_) => {
            valid = false;

            errors.push("Invalid header\n".to_string());
        }
    }

    let mut data: Vec<Vec<String>> = vec![];
    for row in reader.records() {
        match row {
            Ok(r) => {
                let line = r.into_iter().map(|f| f.to_string()).collect();

                data.push(line)
            }
            Err(e) => {
                valid = false;
                errors.push(e.to_string() + "\n")
            }
        }
    }

    verify_telefones(&data, &mut errors, &mut valid);

    let example_data = match data.get(0) {
        Some(first_row) => placeholders
            .clone()
            .into_iter()
            .zip(first_row.clone())
            .collect(),
        None => {
            valid = false;
            errors.push("No rows found\n".to_string());

            HashMap::new()
        }
    };

    let file_data = FileData {
        placeholders,
        destination_count: data.len(),
        example_data,
        valid,
        errors,
    };

    Ok(file_data)
}

pub fn create_csv_data<T: Read>(mut reader: Reader<T>) -> Result<CsvData, String> {
    let placeholders = reader
        .headers()
        .map_err(|_| Err("header_error"))?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    check_header(&placeholders)?;

    let mut data: Vec<Vec<String>> = vec![];
    for row in reader.records() {
        match row {
            Ok(r) => {
                let line = r.into_iter().map(|f| f.to_string()).collect();

                data.push(line)
            }
            Err(_) => return Err(Err("row_format")),
        }
    }

    let csv_data = CsvData { placeholders, data };

    Ok(csv_data)
}

fn check_header(header: &Vec<String>) -> Result<(), String> {
    if header.contains(&String::from("")) {
        return Err(Err("empty_header"));
    };

    let mut buffer = Vec::new();

    for elem in header {
        if buffer.contains(&elem) {
            return Err(Err("duplicate_headers"));
        } else {
            buffer.push(elem);
        }
    }

    Ok(())
}

fn verify_telefones(data: &[Vec<String>], errors: &mut Vec<String>, valid: &mut bool) {
    let regex = Regex::new(r"^\s*\d+\s*$").unwrap();

    let collumns: Vec<String> = data
        .iter()
        .map(|vec| vec.get(0).unwrap_or(&String::new()).to_string())
        .collect();

    for (index, content) in collumns.iter().enumerate() {
        if !regex.is_match(content) {
            *valid = false;
            errors.push(format!(
                "Error on line {}: {} invalid telephone\n",
                index + 2,
                content
            ))
        }
    }
}

pub fn get_separator(line: &str) -> u8 {
    match true {
        _ if line.contains(';') => b';',
        _ if line.contains('|') => b'|',
        _ if line.contains('\t') => b'\t',
        _ if line.contains(',') => b',',
        _ => b',',
    }
}
