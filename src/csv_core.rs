use csv::{Reader, ReaderBuilder};
// use regex::Regex;
// use std::collections::HashMap;
use std::io::Read;
use std::fs::File;
use std::io::{BufReader, BufRead};
use anyhow::{Result, Context, anyhow};


pub struct CsvData {
    pub placeholders: Vec<String>,
    pub data: Vec<Vec<String>>,
}

// pub fn to_text(data: Vec<Vec<String>>) -> Result<String, String> {
//     let mut writer = WriterBuilder::new()
//         .delimiter(b';')
//         .has_headers(true)
//         .from_writer(vec![]);

//     for record in data {
//         writer
//             .write_record(record)
//             .map_err(|_| format!("write_error"))?;
//     }

//     let _ = writer.flush().map_err(|_| return format!("flush_error"));

//     let csv_bin = writer
//         .into_inner()
//         .map_err(|_| format!("inner_erorr"))?;
        
//     let csv_string = String::from_utf8(csv_bin).map_err(|_| format!("stringfy_error"))?;

//     Ok(csv_string)
// }

pub fn path_to_csv_data(path: &str) -> Result<CsvData> {
    let f = File::open(path).with_context(|| format!("failed to open file `{}`", path))?;

    let reader = BufReader::new(f);

    let separator = match reader.lines().next() {
        Some(Ok(first_line)) => get_separator(&first_line),
        Some(Err(e)) => return Err(anyhow!(e)).context("failed to read first line")?,
        None => return Err(anyhow!("separator_error")),
    };

    let csv_reader = ReaderBuilder::new()
        .delimiter(separator)
        .from_path(path)
        .with_context(|| format!("failed to open csv at `{}`", path))?;

    let csv_data = create_csv_data(csv_reader)?;

    Ok(csv_data)
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
