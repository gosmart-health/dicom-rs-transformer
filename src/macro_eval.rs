//! Dynamic macro evaluation engine for generating randomized DICOM values.
//!
//! Supported Macros:
//! - `$uid` / `$UID`: Generate a valid DICOM UID (`2.25.<UUID_u128>`)
//! - `$rand_str(n)` / `$RAND_STR(n)`: Generate random string of uppercase ASCII characters of length `n`
//! - `$rand_num(low, high)` / `$RAND_NUM(low, high)`: Generate a random integer in `[low, high]`
//! - `$rand_time()` / `$RAND_TIME()`: Generate a random DICOM formatted time (`HHMMSS`)
//! - `$today(offset)` / `$TODAY(offset)`: Generate today's date formatted as `YYYYMMDD` +/- offset days
//! - `$$`: Escape sequence evaluating to literal `$`

use chrono::{Duration, Local};
use rand::Rng;
use uuid::Uuid;

use crate::error::TransformError;

/// Evaluates macro expressions within an input template string.
pub fn evaluate_macros(template: &str) -> Result<String, TransformError> {
    if !template.contains('$') {
        return Ok(template.to_string());
    }

    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&'$') = chars.peek() {
                // Escaped $$ -> $
                chars.next();
                result.push('$');
                continue;
            }

            // Read macro identifier name
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_alphanumeric() || nc == '_' {
                    name.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }

            let upper_name = name.to_uppercase();
            match upper_name.as_str() {
                "UID" => {
                    let uid = generate_dicom_uid();
                    result.push_str(&uid);
                }
                "RAND_TIME" => {
                    let mut args = String::new();
                    if let Some(&'(') = chars.peek() {
                        args = read_parenthesized_args(&mut chars)?;
                    }
                    let _ = args;
                    result.push_str(&generate_rand_time());
                }
                "RAND_STR" => {
                    let args = read_parenthesized_args(&mut chars)?;
                    let len: usize = args.trim().parse().map_err(|_| {
                        TransformError::InvalidOperation(format!(
                            "Invalid argument for $rand_str: '{}'",
                            args
                        ))
                    })?;
                    result.push_str(&generate_rand_str(len));
                }
                "RAND_NUM" => {
                    let args = read_parenthesized_args(&mut chars)?;
                    let (low_str, high_str) = args.split_once(',').ok_or_else(|| {
                        TransformError::InvalidOperation(
                            "$rand_num expects two comma-separated numbers (low, high)".to_string(),
                        )
                    })?;
                    let low: i64 = low_str.trim().parse().map_err(|_| {
                        TransformError::InvalidOperation(format!(
                            "Invalid low number '{}'",
                            low_str
                        ))
                    })?;
                    let high: i64 = high_str.trim().parse().map_err(|_| {
                        TransformError::InvalidOperation(format!(
                            "Invalid high number '{}'",
                            high_str
                        ))
                    })?;
                    result.push_str(&generate_rand_num(low, high).to_string());
                }
                "TODAY" => {
                    let mut offset: i64 = 0;
                    if let Some(&'(') = chars.peek() {
                        let args = read_parenthesized_args(&mut chars)?;
                        if !args.trim().is_empty() {
                            offset = args.trim().parse().map_err(|_| {
                                TransformError::InvalidOperation(format!(
                                    "Invalid offset for $today: '{}'",
                                    args
                                ))
                            })?;
                        }
                    }
                    result.push_str(&generate_today(offset));
                }
                _ => {
                    // Unknown macro or single $, preserve as-is
                    result.push('$');
                    result.push_str(&name);
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn read_parenthesized_args(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<String, TransformError> {
    if chars.next() != Some('(') {
        return Err(TransformError::InvalidOperation(
            "Expected '(' for macro arguments".to_string(),
        ));
    }

    let mut args = String::new();
    let mut depth = 1;

    for c in chars.by_ref() {
        match c {
            '(' => {
                depth += 1;
                args.push(c);
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(args);
                }
                args.push(c);
            }
            _ => args.push(c),
        }
    }

    Err(TransformError::InvalidOperation(
        "Unclosed parenthesis in macro arguments".to_string(),
    ))
}

/// Generates a valid ISO OID DICOM UID derived from UUID v4.
pub fn generate_dicom_uid() -> String {
    let uuid = Uuid::new_v4();
    let u128_val = uuid.as_u128();
    format!("2.25.{}", u128_val)
}

/// Generates a random uppercase string of length `len`.
pub fn generate_rand_str(len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..26);
            (b'A' + idx) as char
        })
        .collect()
}

/// Generates a random integer between `low` and `high` inclusive.
pub fn generate_rand_num(low: i64, high: i64) -> i64 {
    let mut rng = rand::thread_rng();
    if low >= high {
        return low;
    }
    rng.gen_range(low..=high)
}

/// Generates a random time string formatted as `HHMMSS`.
pub fn generate_rand_time() -> String {
    let mut rng = rand::thread_rng();
    let hour = rng.gen_range(0..24);
    let min = rng.gen_range(0..60);
    let sec = rng.gen_range(0..60);
    format!("{:02}{:02}{:02}", hour, min, sec)
}

/// Generates a date string `YYYYMMDD` for today +/- `offset_days`.
pub fn generate_today(offset_days: i64) -> String {
    let target = Local::now().naive_local().date() + Duration::days(offset_days);
    target.format("%Y%m%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escaped_dollar() {
        let res = evaluate_macros("Price: $$100").unwrap();
        assert_eq!(res, "Price: $100");
    }

    #[test]
    fn test_uid_macro() {
        let res = evaluate_macros("$uid").unwrap();
        assert!(res.starts_with("2.25."));
    }

    #[test]
    fn test_rand_str_macro() {
        let res = evaluate_macros("PREFIX-$rand_str(8)").unwrap();
        assert_eq!(res.len(), 15);
        assert!(res.starts_with("PREFIX-"));
    }

    #[test]
    fn test_rand_num_macro() {
        let res = evaluate_macros("NUM-$rand_num(10, 20)").unwrap();
        assert!(res.starts_with("NUM-"));
        let num_str = res.strip_prefix("NUM-").unwrap();
        let val: i64 = num_str.parse().unwrap();
        assert!((10..=20).contains(&val));
    }

    #[test]
    fn test_today_macro() {
        let res = evaluate_macros("DATE-$today(0)").unwrap();
        assert_eq!(res.len(), 13); // "DATE-YYYYMMDD"
    }

    #[test]
    fn test_rand_time_macro() {
        let res = evaluate_macros("TIME-$rand_time()").unwrap();
        assert_eq!(res.len(), 11); // "TIME-HHMMSS"
    }
}
