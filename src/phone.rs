use super::countries;
use regex::Regex;

pub fn format_destination(phone_number: &str, country_code: &str) -> String {
    let country_code = country_code.to_uppercase();

    match countries::COUNTRIES
        .iter()
        .map(|c| c.code)
        .any(|x| x == country_code.as_str())
    {
        true => format_as_default(phone_number, &country_code),
        _ => clean_phone_number(phone_number),
    }
}

fn clean_phone_number(phone: &str) -> String {
    let re = Regex::new(r#"[^0-9]"#).unwrap();

    re.replace_all(phone, "").to_string()
}

fn format_as_default(phone: &str, country_code: &str) -> String {
    let clean_phone = clean_phone_number(phone);
    let country_info = get_country_info(country_code);
    let re = Regex::new(&format!("^{}$", country_info.validate_format)).unwrap();

    re.replace(&clean_phone, &country_info.default_format.to_string())
        .to_string()
}

/// Get country info from country code
fn get_country_info<'a>(code: &str) -> countries::CountryInfo<'a> {
    countries::COUNTRIES
        .into_iter()
        .find(|country| country.code == code)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_destination() {
        assert_eq!(format_destination("11 4307 4457", "AR"), "541143074457");
        assert_eq!(format_destination("71234567", "BO"), "59171234567");
        assert_eq!(format_destination("11 97205 7032", "BR"), "5511972057032");
        assert_eq!(format_destination("9 9123 4567", "CL"), "56991234567");
        assert_eq!(format_destination("310 2118879", "CO"), "573102118879");
        assert_eq!(format_destination("83123456", "CR"), "50683123456");
        assert_eq!(format_destination("809-686-5700", "DO"), "18096865700");
        assert_eq!(format_destination("991234567", "EC"), "593991234567");
        assert_eq!(format_destination("71123456", "SV"), "50371123456");
        assert_eq!(format_destination("51234567", "GT"), "50251234567");
        assert_eq!(format_destination("91234567", "HN"), "50491234567");
        assert_eq!(format_destination("55 56173797", "MX"), "525556173797");
        assert_eq!(format_destination("81234567", "NI"), "50581234567");
        assert_eq!(format_destination("61234567", "PA"), "50761234567");
        assert_eq!(format_destination("912345678", "PE"), "51912345678");
        assert_eq!(format_destination("981234567", "PY"), "595981234567");
        assert_eq!(format_destination("7700123456", "UK"), "447700123456");
        assert_eq!(format_destination("623 366 8812", "US"), "16233668812");
        assert_eq!(format_destination("99123456", "UY"), "59899123456");
        assert_eq!(format_destination("412-1234567", "VE"), "584121234567");
    }
}
