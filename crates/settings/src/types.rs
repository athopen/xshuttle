use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A single executable menu item with a display name and command.
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

/// An entry in the menu - either an action or a group.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Entry {
    Action(Action),
    Group(Group),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_serialization() {
        let action = Action {
            name: "Test".to_string(),
            cmd: "echo hello".to_string(),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("echo hello"));
    }

    #[test]
    fn test_group_serialization() {
        let group = Group {
            name: "Production".to_string(),
            entries: vec![Entry::Action(Action {
                name: "Server".to_string(),
                cmd: "ssh prod".to_string(),
            })],
        };
        let json = serde_json::to_string(&group).unwrap();
        assert!(json.contains("Production"));
    }

    #[test]
    fn test_entry_untagged_action() {
        let json = r#"{"name": "Test", "cmd": "echo"}"#;
        let entry: Entry = serde_json::from_str(json).unwrap();
        assert!(matches!(entry, Entry::Action(_)));
    }

    #[test]
    fn test_entry_untagged_group() {
        let json = r#"{"MyGroup": [{"name": "Test", "cmd": "echo"}]}"#;
        let entry: Entry = serde_json::from_str(json).unwrap();
        assert!(matches!(entry, Entry::Group(_)));
    }
}
