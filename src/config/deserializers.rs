// Simple Forgejo instance guardian, banning users and alerting admins based on
// certain regular expressions. Copyright (C) 2024 Awiteb <a@4rs.nl>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://gnu.org/licenses/agpl.txt>.

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
