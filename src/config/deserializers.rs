// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024-2025 Awiteb <a@4rs.nl>

use regex::Regex;
use serde::{de, Deserialize};
use toml::Value;
use url::Url;

use super::{utils, RegexReason};

/// Deserialize a string into a `url::Url`
///
/// This will check if the url is `http` or `https` and if it is a valid url
pub fn url<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
    D: de::Deserializer<'de>,
{
    let url =
        Url::parse(&String::deserialize(deserializer)?).map_err(utils::custom_de_err::<'de, D>)?;
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(de::Error::custom("URL scheme must be http or https"));
    }
    Ok(url)
}

/// Parse the `re` key in the table, which can be a string or an array of string
fn parse_re<'de, D>(toml_value: &Value) -> Result<Vec<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    match toml_value {
        Value::String(str_re) => Ok(vec![str_re.to_owned()]),
        Value::Array(re_vec) => {
            re_vec
                .iter()
                .map(|str_re| {
                    str_re.as_str().map(String::from).ok_or_else(|| {
                        <D::Error as de::Error>::custom(format!(
                            "expected an array of string, found `{str_re}`"
                        ))
                    })
                })
                .collect()
        }
        value => {
            Err(<D::Error as de::Error>::custom(format!(
                "expected a string value or an array of string for `re`, found `{value}`"
            )))
        }
    }
}

/// Parse the vector of string regex to `Vec<Regex>`
fn parse_re_vec<'de, D>(re_vec: Vec<String>) -> Result<Vec<Regex>, D::Error>
where
    D: de::Deserializer<'de>,
{
    re_vec
        .into_iter()
        .map(|re| re.parse().map_err(utils::custom_de_err::<'de, D>))
        .collect()
}

/// Deserialize `RegexReason`
pub fn regex_reason<'de, D>(deserializer: D) -> Result<Vec<RegexReason>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let Ok(toml_value) = Vec::<Value>::deserialize(deserializer) else {
        return Err(de::Error::custom(
            "expected an array contains strings or arrays of string or tables with the keys `re` \
             and optional `reason`",
        ));
    };

    toml_value
        .into_iter()
        .map(|value| {
            if let Value::Table(table) = value {
                let re_vec = table.get("re").map(parse_re::<D>).ok_or_else(|| {
                    <D::Error as de::Error>::custom(
                        "The table must contain a `re` key with a string value or an array of \
                         string",
                    )
                })??;
                let reason = table
                    .get("reason")
                    .map(|reason| {
                        reason.as_str().map(String::from).ok_or_else(|| {
                            <D::Error as de::Error>::custom(format!(
                                "expected a string value for `reason`, found `{reason}`"
                            ))
                        })
                    })
                    .transpose()?;

                // Warn for unused keys
                for key in table.keys() {
                    if !["re", "reason"].contains(&key.as_str()) {
                        tracing::warn!("Unused key `{key}` in the configuration");
                    }
                }

                Ok(RegexReason::new(parse_re_vec::<D>(re_vec)?, reason))
            } else if matches!(value, Value::String(_) | Value::Array(_)) {
                Ok(RegexReason::new(
                    parse_re_vec::<D>(parse_re::<D>(&value)?)?,
                    None,
                ))
            } else {
                Err(de::Error::custom(format!(
                    "unexpected value in the regex list, expected a string or an array of string \
                     or a table with `re` (string) and optional `reason` (string), found `{value}`"
                )))
            }
        })
        .collect()
}

pub fn prefixed_interval<'de, D>(des: D) -> Result<u32, D::Error>
where
    D: de::Deserializer<'de>,
{
    let interval = String::deserialize(des)
        .map_err(|_| de::Error::custom("Expected a prefixed interval, e.g. 1s, 2m"))?;
    if interval.chars().count() < 2 {
        return Err(de::Error::custom(format!(
            "Expected a prefixed interval, e.g. 1s, 2m. found {interval}"
        )));
    }
    let mut chars = interval.chars();
    chars.next_back();
    let number = chars.as_str().parse::<u32>().map_err(|err| {
        let msg = match err.kind() {
            std::num::IntErrorKind::PosOverflow => {
                format!("Number is too large: {}", chars.as_str())
            }
            _ => format!("Expected positive integer, found: {}", chars.as_str()),
        };

        de::Error::custom(msg)
    })?;
    let prefix = interval.chars().last().expect("the length more then 2");
    let interval = match prefix {
        's' => number,
        'm' => number * 60,
        'h' => number * 60 * 60,
        'd' => number * 24 * 60 * 60,
        _ => {
            return Err(de::Error::custom(format!(
                "Unknown prefix `{prefix}`, expected s, m, h, d"
            )))
        }
    };

    Ok(interval)
}
