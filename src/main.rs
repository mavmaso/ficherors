use std::collections::HashMap;
use csv::{WriterBuilder};
use anyhow::{Result, anyhow};

mod countries;
mod csv_core;
mod functions;
mod phone;

type Maps = HashMap<String, HashMap<String, String>>;

fn main() {
    println!("Running ...");
    let res = process_csv("20mb.txt", "BR", HashMap::new()).unwrap();
    println!("Finished");
    println!("{}", res);
}

fn process_csv(path: &str, country_code: &str, functions: Maps) -> Result<String> {
    let csv_data = csv_core::path_to_csv_data(path).map_err(|e| anyhow!(e))?;
    let with_functions = functions != HashMap::new();
    let old_headers = csv_data.placeholders.clone();
    let mut leftover_headers = vec![];
    let mut header = vec!["d3stinati0n".to_string()];
    let new_path = "destinations.csv".to_string();

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

    let mut writer = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
    .from_path(&new_path).map_err(|e| anyhow!(e))?;

    writer.write_record(header).map_err(|e| anyhow!(e))?;

    for line in csv_data.data {
        let phone_number = line.clone().remove(0);
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

        writer.write_record(new_row).map_err(|e| anyhow!(e))?;
    }

    Ok(new_path)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_process_csv() {
        let result = process_csv("0mb.txt", "BR", HashMap::new());
        assert!(result.is_ok(), "process_csv should return Ok with valid input");
    }
}
