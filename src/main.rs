use std::collections::HashMap;

mod countries;
mod csv_core;
mod functions;
mod phone;

// use csv_core::CsvData;

type Maps = HashMap<String, HashMap<String, String>>;


fn main() {
    println!("Running ...");
    let _res = process_csv("1giga.csv", "BR", HashMap::new()).unwrap();
    println!("Finished");

    // dbg!(res);
}

// fn csv_reader(path: &str) -> Result<CsvData, String> {
//     let csv_data = csv_core::path_to_csv_data(path)?;

//     Ok(csv_data)
// }

// fn csv_to_text(data: Vec<Vec<String>>) -> Result<String, String> {
//     csv_core::to_text(data)
// }

fn process_csv(path: &str, country_code: &str, functions: Maps) -> Result<String, String> {
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

    Ok(csv_core::to_text(body)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_process_csv() {
        let result = process_csv("1giga.csv", "BR", HashMap::new());
        assert!(result.is_ok(), "process_csv should return Ok with valid input");
    }
}
