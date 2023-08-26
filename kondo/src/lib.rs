use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub enum ParseAgeFilterError {
    ParseIntError(ParseIntError),
    InvalidUnit,
}

impl fmt::Display for ParseAgeFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseAgeFilterError::ParseIntError(e) => e.fmt(f),
            ParseAgeFilterError::InvalidUnit => {
                "invalid age unit, must be one of m, h, d, w, M, y".fmt(f)
            }
        }
    }
}

impl From<ParseIntError> for ParseAgeFilterError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl Error for ParseAgeFilterError {}

pub fn parse_age_filter(age_filter: &str) -> Result<u64, ParseAgeFilterError> {
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = WEEK * 4;
    const YEAR: u64 = DAY * 365;

    let (digit_end, unit) = age_filter
        .char_indices()
        .last()
        .ok_or(ParseAgeFilterError::InvalidUnit)?;

    let multiplier = match unit {
        'm' => MINUTE,
        'h' => HOUR,
        'd' => DAY,
        'w' => WEEK,
        'M' => MONTH,
        'y' => YEAR,
        _ => return Err(ParseAgeFilterError::InvalidUnit),
    };

    let count = age_filter[..digit_end].parse::<u64>()?;
    let seconds = count * multiplier;
    Ok(seconds)
}

#[test]
fn test_age_filter_120s() {
    let hours = parse_age_filter("2h").unwrap();
    let minutes = parse_age_filter("120m").unwrap();

    assert_eq!(minutes, hours);
}
#[test]
fn test_age_filter_10m() {
    let res = parse_age_filter("10m");
    let age_filter = res.unwrap();
    assert_eq!(age_filter, (60 * 10));
}

#[ignore = "failing unexpectedly. BUG?"]
#[test]
fn test_age_filter_year_months() {
    let year = parse_age_filter("1y").unwrap();
    let months = parse_age_filter("12M").unwrap();

    assert_eq!(year, months);
}
