use anyhow::anyhow;
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct NonEmptyString {
    inner: String,
}
impl NonEmptyString {
    pub fn new(s: &str) -> anyhow::Result<Self> {
        if s.is_empty() {
            return Err(anyhow!(
                "cannot construct NonEmptyString from an empty &str"
            ));
        }

        Ok(Self { inner: s.into() })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}
impl std::fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.inner.clone()
    }
}
impl TryFrom<String> for NonEmptyString {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        NonEmptyString::new(&value)
    }
}
impl Serialize for NonEmptyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NonEmptyStringVisitor;

        impl<'de> Visitor<'de> for NonEmptyStringVisitor {
            type Value = NonEmptyString;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a non-empty string")
            }

            fn visit_str<E>(self, value: &str) -> Result<NonEmptyString, E>
            where
                E: serde::de::Error,
            {
                if value.is_empty() {
                    Err(serde::de::Error::custom("string cannot be empty"))
                } else {
                    Ok(NonEmptyString {
                        inner: value.to_owned(),
                    })
                }
            }
        }

        deserializer.deserialize_string(NonEmptyStringVisitor)
    }
}
