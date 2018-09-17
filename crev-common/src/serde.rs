// Oh dear, this module is called serde, and is in the root
// so we have to import serde crate here
extern crate serde;

use self::serde::Deserialize;
use base64;
use chrono::{self, offset::FixedOffset, prelude::*};
use hex::{self, FromHex, FromHexError};
use serde_yaml;
use std::{fmt, io};

// {{{ Serde serialization
pub trait MyTryFromBytes: Sized {
    type Err: 'static + Sized + ::std::error::Error;
    fn try_from(_: &[u8]) -> Result<Self, Self::Err>;
}

pub fn from_base64<'d, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'d>,
    T: MyTryFromBytes,
{
    use self::serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| {
            base64::decode_config(&string, base64::URL_SAFE)
                .map_err(|err| Error::custom(err.to_string()))
        }).and_then(|ref bytes| {
            T::try_from(bytes)
                .map_err(|err| Error::custom(format!("{}", &err as &::std::error::Error)))
        })
}

pub fn as_base64<T, S>(key: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: serde::Serializer,
{
    serializer.serialize_str(&base64::encode_config(key.as_ref(), base64::URL_SAFE))
}

pub fn from_hex<'d, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'d>,
    T: MyTryFromBytes,
{
    use self::serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| {
            FromHex::from_hex(string.as_str())
                .map_err(|err: FromHexError| Error::custom(err.to_string()))
        }).and_then(|bytes: Vec<u8>| {
            T::try_from(&bytes)
                .map_err(|err| Error::custom(format!("{}", &err as &::std::error::Error)))
        })
}

pub fn as_hex<T, S>(key: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: serde::Serializer,
{
    serializer.serialize_str(&hex::encode(key))
}

pub fn from_rfc3339_fixed<'d, D>(deserializer: D) -> Result<chrono::DateTime<FixedOffset>, D::Error>
where
    D: serde::Deserializer<'d>,
{
    use self::serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| {
            DateTime::<FixedOffset>::parse_from_rfc3339(&string)
                .map_err(|err| Error::custom(err.to_string()))
        }).map(|dt| dt.with_timezone(&dt.timezone()))
}

pub fn as_rfc3339_fixed<S>(
    key: &chrono::DateTime<FixedOffset>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&key.to_rfc3339())
}

impl MyTryFromBytes for Vec<u8> {
    type Err = io::Error;
    fn try_from(slice: &[u8]) -> Result<Self, Self::Err> {
        Ok(Vec::from(slice))
    }
}
// }}}

/// Write out a value as YAML without a `---` prefix
///
/// This is how a lot of stuff in `Crev` is serialized
pub fn write_as_headerless_yaml<T: self::serde::Serialize>(
    t: &T,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    // TODO: Don't serialize to string, and instead serialize to writer
    let yaml_document = serde_yaml::to_string(t).map_err(|_| fmt::Error)?;
    let mut lines = yaml_document.lines();
    let dropped_header = lines.next();
    assert_eq!(dropped_header, Some("---"));

    for line in lines {
        f.write_str(&line)?;
        f.write_str("\n")?;
    }
    Ok(())
}