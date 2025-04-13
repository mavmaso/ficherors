use csv::{Reader, ReaderBuilder, WriterBuilder};
// use regex::Regex;
// use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::io::{BufReader, BufRead};


pub struct CsvData {
    pub placeholders: Vec<String>,
    pub data: Vec<Vec<String>>,
}

pub fn to_text(data: Vec<Vec<String>>) -> Result<String, String> {
    let mut writer = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_writer(vec![]);

    for record in data {
        writer
            .write_record(record)
            .map_err(|_| format!("write_error"))?;
    }

    let _ = writer.flush().map_err(|_| return format!("flush_error"));

    let csv_bin = writer
        .into_inner()
        .map_err(|_| format!("inner_erorr"))?;
        
    let csv_string = String::from_utf8(csv_bin).map_err(|_| format!("stringfy_error"))?;

    Ok(csv_string)
}

pub fn path_to_csv_data(path: &str) -> Result<CsvData, String> {
    let file = File::open(path).map_err(|_| format!("file_not_found"))?;

    let reader = BufReader::new(file);

    let separator = match reader.lines().next() {
        Some(Ok(first_line)) => get_separator(&first_line),
        _ => return  Err(format!("separator_error")),
    };

    let file = ReaderBuilder::new()
        .delimiter(separator)
        .from_path(path)
        .map_err(|_| format!("csv_not_found"))?;

    let csv_data = create_csv_data(file)?;

    Ok(csv_data)
}

pub fn create_csv_data<T: Read>(mut reader: Reader<T>) -> Result<CsvData, String> {
    let placeholders = reader
        .headers()
        .map_err(|_| format!("header_error"))?
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
            Err(_) => return Err(format!("row_format")),
        }
    }

    let csv_data = CsvData { placeholders, data };

    Ok(csv_data)
}

fn check_header(header: &Vec<String>) -> Result<(), String> {
    if header.contains(&String::from("")) {
        return Err(format!("empty_header"));
    };

    let mut buffer = Vec::new();

    for elem in header {
        if buffer.contains(&elem) {
            return Err(format!("duplicate_headers"));
        } else {
            buffer.push(elem);
        }
    }

    Ok(())
}

// fn verify_telefones(data: &[Vec<String>], errors: &mut Vec<String>, valid: &mut bool) {
//     let regex = Regex::new(r"^\s*\d+\s*$").unwrap();

//     let collumns: Vec<String> = data
//         .iter()
//         .map(|vec| vec.get(0).unwrap_or(&String::new()).to_string())
//         .collect();

//     for (index, content) in collumns.iter().enumerate() {
//         if !regex.is_match(content) {
//             *valid = false;
//             errors.push(format!(
//                 "Error on line {}: {} invalid telephone\n",
//                 index + 2,
//                 content
//             ))
//         }
//     }
// }

pub fn get_separator(line: &str) -> u8 {
    match true {
        _ if line.contains(';') => b';',
        _ if line.contains('|') => b'|',
        _ if line.contains('\t') => b'\t',
        _ if line.contains(',') => b',',
        _ => b',',
    }
}
