use calamine::{open_workbook, DataType, Reader, Xlsx};
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

mod countries;
mod csv_core;
mod functions;
mod phone;

use csv_core::CsvData;
use csv_core::FileData;

type Maps = HashMap<String, HashMap<String, String>>;


fn csv_reader(path: String) -> NifResult<(Atom, CsvData)> {
    let file = File::open(&path).map_err(|_| Error::Atom("file_not_found"))?;

    let reader = BufReader::new(file);

    let separator = match reader.lines().next() {
        Some(Ok(first_line)) => csv_core::get_separator(&first_line),
        _ => return Err(Error::Atom("separator_error")),
    };

    let file = ReaderBuilder::new()
        .delimiter(separator)
        .from_path(&path)
        .map_err(|_| Error::Atom("csv_not_found"))?;

    let csv_data = csv_core::create_csv_data(file)?;

    Ok((ok(), csv_data))
}


fn excel_reader(path: String) -> NifResult<(Atom, String)> {
    let mut excel: Xlsx<_> = open_workbook(path).map_err(|_| Error::Atom("file_not_found"))?;
    let mut content = String::new();

    match excel.worksheet_range_at(0) {
        Some(Ok(sheet)) => {
            for row in sheet.rows() {
                for cell in row.iter() {
                    match cell {
                        DataType::String(string) => content.push_str(string),
                        DataType::Float(float) => content.push_str(&float.to_string()),
                        _ => content.push_str(""),
                    }

                    content.push(';');
                }

                content.pop();
                content.push('\n')
            }
        }
        _ => return Err(Error::Atom("worksheet_not_found")),
    }

    Ok((ok(), content))
}


fn csv_content_reader(content: &str) -> NifResult<(Atom, FileData)> {
    let file_data = csv_core::verify_content(content)?;

    Ok((ok(), file_data))
}


fn process_csv(path: &str, country_code: &str, functions: Maps) -> NifResult<(Atom, String)> {
    let csv_data = csv_core::path_to_csv_data(path)?;
    let with_functions = functions != HashMap::new();
    let old_headers = csv_data.placeholders.clone().split_off(1);
    let mut leftover_headers = vec![];
    let mut header = vec!["d3stinati0n".to_string()];

    if with_functions {
        functions.keys().for_each(|k| header.push(k.to_string()));
        let keys: Vec<String> = functions.keys().map(|k| k.to_string()).collect();

        old_headers
            .iter()
            .filter(|i| !keys.contains(i))
            .for_each(|filtered| leftover_headers.push(filtered.to_string()));
        
        header.extend(leftover_headers.clone())
    } else {
        header.extend(old_headers.clone())
    }

    let mut body: Vec<Vec<String>> = vec![];
    body.push(header);

    for mut line in csv_data.data {
        let phone_number = line.remove(0);
        let mut new_row = vec![];

        new_row.push(phone::format_destination(&phone_number, country_code));

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

        body.push(new_row);
    }

    Ok((ok(), csv_core::to_text(body)?))
}

