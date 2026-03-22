use calamine::{open_workbook, Data, Reader, Xlsx};
use chrono::{Duration, NaiveDate};
use anyhow::{anyhow, Result};

/// Reads an Excel (.xlsx) file and returns its content as a semicolon-separated string.
pub fn path_to_text(path: &str) -> Result<String> {
    let mut excel: Xlsx<_> = open_workbook(path).map_err(|_| anyhow!("file_not_found"))?;
    let mut content = String::new();

    match excel.worksheet_range_at(0) {
        Some(Ok(sheet)) => {
            for row in sheet.rows() {
                for cell in row.iter() {
                    match *cell {
                        Data::Empty => (),
                        Data::String(ref s)
                        | Data::DateTimeIso(ref s)
                        | Data::DurationIso(ref s) => content.push_str(s),
                        Data::Float(ref f) => content.push_str(&f.to_string()),
                        Data::DateTime(ref d) => {
                            let s = convert_excel_date_time(d.as_f64())?;
                            content.push_str(&s);
                        }
                        Data::Int(ref i) => content.push_str(&i.to_string()),
                        Data::Bool(ref b) => content.push_str(&b.to_string()),
                        Data::Error(_) => (),
                    };
                    content.push(';');
                }
                content.pop(); // remove trailing ';'
                content.push('\n');
            }
        }
        _ => return Err(anyhow!("worksheet_not_found")),
    }

    Ok(content)
}

fn convert_excel_date_time(days: f64) -> Result<String> {
    if days < 1.0 {
        let total_hours = days * 24.0;
        let hours = total_hours.floor() as u32;
        let minutes = ((total_hours - hours as f64) * 60.0).round() as u32;

        if minutes == 60 {
            return Ok(format!("{:02}:{:02}", hours + 1, 0));
        }

        Ok(format!("{:02}:{:02}", hours, minutes))
    } else {
        let duration: i64 = days.round() as i64;
        let start = NaiveDate::from_ymd_opt(1899, 12, 30).expect("DATE");

        match start.checked_add_signed(Duration::days(duration)) {
            Some(v) => Ok(v.format("%d/%m/%Y").to_string()),
            None => Err(anyhow!("date_time_error")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::convert_excel_date_time;

    #[test]
    fn test_convert_excel_date_time() {
        let less_than_one = convert_excel_date_time(0.90);
        assert!(matches!(less_than_one, Ok(ref s) if s == "21:36"));

        let more_than_one = convert_excel_date_time(1.0);
        assert!(matches!(more_than_one, Ok(ref s) if s == "31/12/1899"));

        let date_in_2025 = convert_excel_date_time(45700.0);
        assert!(matches!(date_in_2025, Ok(ref s) if s == "12/02/2025"));
    }
}
