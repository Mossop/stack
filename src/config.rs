use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::Read;

use serde::de::{self, Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_with::formats::SpaceSeparator;
use serde_with::{serde_as, StringWithSeparator};

fn deserialize_file<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FileVisitor {}
    impl<'de> Visitor<'de> for FileVisitor {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a list of strings")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Option<Vec<String>>, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut list = Vec::new();
            while let Some(s) = seq.next_element()? {
                list.push(s);
            }
            Ok(Some(list))
        }

        fn visit_str<E>(self, s: &str) -> Result<Option<Vec<String>>, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![s.to_owned()]))
        }

        fn visit_none<E>(self) -> Result<Option<Vec<String>>, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(FileVisitor {})
}

#[serde_as]
#[derive(Deserialize)]
pub struct Stack {
    #[serde(skip)]
    pub key: String,
    #[serde(default)]
    pub name: String,
    pub directory: Option<String>,
    #[serde(default, deserialize_with = "deserialize_file")]
    pub file: Option<Vec<String>>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

fn default_command() -> Vec<String> {
    vec!["docker".to_string(), "compose".to_string()]
}

fn check_dependencies(
    stack: &Stack,
    stacks: &BTreeMap<String, Stack>,
    seen: &mut HashSet<String>,
) -> Result<(), String> {
    seen.insert(stack.key.clone());

    for dep in stack.depends_on.iter() {
        if seen.contains(dep) {
            return Err(format!(
                "invalid dependency cycle: \"{}\" cannot depend on \"{}\"",
                stack.key, dep
            ));
        }

        if let Some(inner) = stacks.get(dep) {
            check_dependencies(inner, stacks, seen)?;
        } else {
            return Err(format!(
                "invalid dependency: \"{}\" is not a known stack",
                dep
            ));
        }
    }

    seen.remove(&stack.key);

    Ok(())
}

fn deserialize_stacks<'de, D>(deserializer: D) -> Result<BTreeMap<String, Stack>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut stacks: BTreeMap<String, Stack> = BTreeMap::deserialize(deserializer)?;

    for (key, stack) in stacks.iter_mut() {
        stack.key = key.clone();
        if stack.name.is_empty() {
            stack.name = key.clone();
        }
    }

    let mut seen = HashSet::new();
    for stack in stacks.values() {
        check_dependencies(stack, &stacks, &mut seen).map_err(D::Error::custom)?;
    }

    Ok(stacks)
}

#[serde_as]
#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_command")]
    #[serde_as(as = "StringWithSeparator::<SpaceSeparator, String>")]
    pub command: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_stacks")]
    pub stacks: BTreeMap<String, Stack>,
}

fn add_deps(stacks: &BTreeMap<String, Stack>, stack: &str, keys: &mut BTreeSet<String>) {
    let stack = stacks.get(stack).unwrap();

    for dep in stack.depends_on.iter() {
        if keys.contains(dep) {
            continue;
        }
        keys.insert(dep.to_owned());

        add_deps(stacks, dep, keys);
    }
}

impl Config {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, String> {
        serde_yaml::from_reader(reader).map_err(|e| e.to_string())
    }

    pub fn stacks<I, S>(&self, list: I, with_dependencies: bool) -> Result<Vec<&Stack>, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        // First list all the desired stacks. Use an ordered set here so results
        // are stable.
        let mut keys: BTreeSet<String> = BTreeSet::new();

        for key in list {
            let key_str = key.as_ref();

            if self.stacks.contains_key(key_str) {
                keys.insert(key_str.to_owned());
            } else {
                return Err(format!("unknown stack \"{}\"", key_str));
            }
        }

        // An empty initial list means we want all stacks.
        if keys.is_empty() {
            keys = self.stacks.keys().map(|k| k.to_owned()).collect();
        } else if with_dependencies {
            // Add all dependencies if needed.
            if with_dependencies {
                for key in keys.clone().iter() {
                    add_deps(&self.stacks, key, &mut keys);
                }
            }
        }

        // Now to order them in dependent order.
        let mut added: HashSet<String> = HashSet::new();
        let mut stacks = Vec::new();

        while stacks.len() != keys.len() {
            for key in keys.iter().rev() {
                if added.contains(key) {
                    // Already added this one.
                    continue;
                }

                let stack = self.stacks.get(key).unwrap();
                if stack
                    .depends_on
                    .iter()
                    .all(|dep: &String| added.contains(dep) || !keys.contains(dep))
                {
                    // All dependencies we care about are resolved so this stack
                    // can be added.
                    stacks.insert(0, stack);
                    added.insert(key.to_owned());
                }
            }
        }

        Ok(stacks)
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, Stack};

    fn from_str(s: &str) -> Result<Config, String> {
        Config::from_reader(s.as_bytes())
    }

    fn keys(stacks: Vec<&Stack>) -> Vec<String> {
        stacks.iter().map(|s| s.key.clone()).collect()
    }

    #[test]
    fn name() {
        let config = from_str(
            "
            stacks:
                foo:
                    name: baz
                bar: {}
            ",
        )
        .unwrap();

        let stacks = config.stacks(["foo", "bar"], false).unwrap();
        assert_eq!(stacks.len(), 2);
        let stack = stacks.get(0).unwrap();
        assert_eq!(stack.name, "bar");
        let stack = stacks.get(1).unwrap();
        assert_eq!(stack.name, "baz");
    }

    #[test]
    fn stacks() {
        let config = from_str(
            "
            stacks:
                foo:
                    depends_on:
                        - bar
                bar:
                    depends_on:
                        - baz
                baz: {}
            ",
        )
        .unwrap();
        let list: [&str; 0] = [];
        let stacks = keys(config.stacks(list, false).unwrap());
        assert_eq!(
            stacks,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );

        let stacks = keys(config.stacks(["bar"], false).unwrap());
        assert_eq!(stacks, vec!["bar".to_string()]);

        let stacks = keys(config.stacks(["bar"], true).unwrap());
        assert_eq!(stacks, vec!["bar".to_string(), "baz".to_string()]);

        let stacks = keys(config.stacks(["foo"], true).unwrap());
        assert_eq!(
            stacks,
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );

        assert_eq!(
            config.stacks(["bar", "biz"], false).err().unwrap(),
            "unknown stack \"biz\""
        );

        let error = from_str(
            "
            stacks:
                foo:
                    depends_on:
                        - baz
            ",
        )
        .err()
        .unwrap();
        assert_eq!(
            &error,
            "invalid dependency: \"baz\" is not a known stack at line 2 column 13"
        );

        let error = from_str(
            "
            stacks:
                foo:
                    depends_on:
                        - baz
                baz:
                    depends_on:
                        - foo
            ",
        )
        .err()
        .unwrap();
        assert_eq!(
            &error,
            "invalid dependency cycle: \"foo\" cannot depend on \"baz\" at line 2 column 13"
        );

        let error = from_str(
            "
            stacks:
                foo:
                    depends_on:
                        - bar
                baz:
                    depends_on:
                        - foo
                bar:
                    depends_on:
                        - baz
            ",
        )
        .err()
        .unwrap();
        assert_eq!(
            &error,
            "invalid dependency cycle: \"foo\" cannot depend on \"bar\" at line 2 column 13"
        );

        let config = from_str(
            "
            stacks:
                foo:
                    depends_on:
                        - bar
                bar:
                    depends_on:
                        - baz
                        - biz
                biz:
                    depends_on:
                        - baz
                baz: {}
            ",
        )
        .unwrap();
        let stacks = keys(config.stacks(["foo"], true).unwrap());
        assert_eq!(
            stacks,
            vec![
                "foo".to_string(),
                "bar".to_string(),
                "biz".to_string(),
                "baz".to_string()
            ]
        );

        let stacks = keys(config.stacks(["bar"], false).unwrap());
        assert_eq!(stacks, vec!["bar".to_string()]);
    }
}
