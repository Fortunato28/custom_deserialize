use anyhow::Result;
use de::MapAccess;
use itertools::Itertools;
use regex::Regex;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Id {
    pub exchange_id: String,

    pub account_number: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdParseError(String);

impl Id {
    pub fn new(exchange_id: String, account_number: u8) -> Self {
        Self {
            exchange_id,
            account_number,
        }
    }

    pub fn default() -> Self {
        Self {
            exchange_id: "test".to_owned(),
            account_number: 0,
        }
    }
}

impl FromStr for Id {
    type Err = IdParseError;

    fn from_str(text: &str) -> std::result::Result<Self, Self::Err> {
        let regex =
            Regex::new(r"(^[A-Za-z]+)(\d+$)").map_err(|err| IdParseError(err.to_string()))?;

        let captures = regex
            .captures(text)
            .ok_or(IdParseError("Invalid format".into()))?
            .iter()
            .collect_vec();

        let exchange_id = captures[1]
            .ok_or(IdParseError("Invalid format".into()))?
            .as_str()
            .into();

        let number = captures[2]
            .ok_or(IdParseError("Invalid format".into()))?
            .as_str()
            .parse()
            .map_err(|x| IdParseError(format!("Can't parse exchange account number: {}", x)))?;

        Ok(Id::new(exchange_id, number))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Id, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            ExchangeAccountId,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("field exchange_id or account_number")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "exchange_id" => Ok(Field::ExchangeAccountId),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string with unsigned integer on the tail")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Id, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut whole_field = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ExchangeAccountId => {
                            if whole_field.is_some() {
                                return Err(de::Error::duplicate_field("exchange_id"));
                            }
                            whole_field = Some(map.next_value()?);
                        }
                    }
                }
                let whole_field: String =
                    whole_field.ok_or_else(|| de::Error::missing_field("exchange_id"))?;

                Id::from_str(&whole_field)
                    .map_err(|_| de::Error::unknown_field(&whole_field, FIELDS))
            }
        }

        const FIELDS: &'static [&'static str] = &["exchange_id", "account_number"];
        deserializer.deserialize_struct("Id", FIELDS, IdVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Settings {
    pub test: i32,
    pub exchange_id: Id,
}

fn main() -> Result<()> {
    let mut config = config::Config::default();
    config.merge(config::File::with_name("config.toml"))?;

    let deserialized: Settings = config.try_into()?;
    dbg!(&deserialized);
    println!("Hello, world!");

    Ok(())
}
