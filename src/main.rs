use anyhow::Result;
use de::MapAccess;
use serde::de::{self, Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Id {
    pub exchange_id: String,

    pub account_number: u8,
}

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

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Id, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            ExchangeId,
            AccountNumber,
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
                            "exchange_id" => Ok(Field::ExchangeId),
                            "account_number" => Ok(Field::AccountNumber),
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
                let mut exchange_id = None;
                let mut account_number = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ExchangeId => {
                            if exchange_id.is_some() {
                                return Err(de::Error::duplicate_field("exchange_id"));
                            }
                            exchange_id = Some(map.next_value()?);
                        }
                        Field::AccountNumber => {
                            if account_number.is_some() {
                                return Err(de::Error::duplicate_field("account_number"));
                            }
                            account_number = Some(map.next_value()?);
                        }
                    }
                }
                let exchange_id =
                    exchange_id.ok_or_else(|| de::Error::missing_field("exchange_id"))?;
                let account_number =
                    account_number.ok_or_else(|| de::Error::missing_field("account_number"))?;

                dbg!(&exchange_id);
                dbg!(&account_number);

                Ok(Id::new(exchange_id, account_number))
            }
        }

        const FIELDS: &'static [&'static str] = &["exchange_id", "account_number"];
        deserializer.deserialize_struct("Id", FIELDS, IdVisitor)
    }
}

fn main() -> Result<()> {
    let mut config = config::Config::default();
    config.merge(config::File::with_name("config.toml"))?;

    let deserialized: Id = config.try_into()?;
    dbg!(&deserialized);
    println!("Hello, world!");

    Ok(())
}
