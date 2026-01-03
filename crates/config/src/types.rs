use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    pub name: String,
    pub cmd: String,
}

/// A named group containing nested entries.
/// Serializes to/from JSON as `{"GroupName": [...]}`
#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub entries: Vec<Entry>,
}

impl Serialize for Group {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.name, &self.entries)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Group {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GroupVisitor;

        impl<'de> Visitor<'de> for GroupVisitor {
            type Value = Group;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with a single key (group name) and array value")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let (name, entries): (String, Vec<Entry>) = access
                    .next_entry()?
                    .ok_or_else(|| de::Error::custom("expected non-empty map for group"))?;

                // Ensure no extra keys
                if access.next_key::<String>()?.is_some() {
                    return Err(de::Error::custom(
                        "group must have exactly one key (the group name)",
                    ));
                }

                Ok(Group { name, entries })
            }
        }

        deserializer.deserialize_map(GroupVisitor)
    }
}

/// An entry in the menu - either a action or a group.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Action(Action),
    Group(Group),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_terminal")]
    pub terminal: String,

    #[serde(default = "default_editor")]
    pub editor: String,

    #[serde(rename = "actions", default)]
    pub entries: Vec<Entry>,
}

fn default_terminal() -> String {
    "default".to_string()
}

fn default_editor() -> String {
    "default".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: default_terminal(),
            editor: default_editor(),
            entries: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.terminal, "default");
        assert_eq!(config.editor, "default");
        assert!(config.entries.is_empty());
    }
}
