#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConfigKeySchema {
    pub name: &'static str,
    pub default: usize,
    pub description: &'static str,
    pub min: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConfigSectionSchema {
    pub name: &'static str,
    pub keys: &'static [ConfigKeySchema],
}

const LIMITS_KEYS: [ConfigKeySchema; 2] = [
    ConfigKeySchema {
        name: "todo",
        default: 20,
        description: "max todo tasks",
        min: 1,
    },
    ConfigKeySchema {
        name: "in_progress",
        default: 3,
        description: "max in-progress tasks",
        min: 1,
    },
];

const HINTS_KEYS: [ConfigKeySchema; 1] = [ConfigKeySchema {
    name: "learn_threshold",
    default: 35,
    description: "show `lt learn` hint after this many LEARNINGS.md lines",
    min: 1,
}];

const RETENTION_KEYS: [ConfigKeySchema; 1] = [ConfigKeySchema {
    name: "done_discard_ttl_days",
    default: 7,
    description: "auto-delete done/discard tasks older than this many days",
    min: 1,
}];

pub(crate) const USER_CONFIG_SCHEMA: [ConfigSectionSchema; 3] = [
    ConfigSectionSchema {
        name: "limits",
        keys: &LIMITS_KEYS,
    },
    ConfigSectionSchema {
        name: "hints",
        keys: &HINTS_KEYS,
    },
    ConfigSectionSchema {
        name: "retention",
        keys: &RETENTION_KEYS,
    },
];

pub(crate) fn key_schema(section: &str, key: &str) -> Option<&'static ConfigKeySchema> {
    USER_CONFIG_SCHEMA
        .iter()
        .find(|candidate| candidate.name == section)
        .and_then(|candidate| candidate.keys.iter().find(|entry| entry.name == key))
}

pub(crate) fn default_usize(section: &str, key: &str) -> usize {
    key_schema(section, key)
        .map(|entry| entry.default)
        .unwrap_or_else(|| panic!("missing schema entry for {section}.{key}"))
}

pub(crate) fn min_usize(section: &str, key: &str) -> usize {
    key_schema(section, key)
        .map(|entry| entry.min)
        .unwrap_or_else(|| panic!("missing schema entry for {section}.{key}"))
}

pub(crate) fn render_default_config_body() -> String {
    let mut body = String::new();

    for (section_index, section) in USER_CONFIG_SCHEMA.iter().enumerate() {
        if section_index > 0 {
            body.push('\n');
        }
        body.push_str(&format!("[{}]\n", section.name));
        for key in section.keys {
            body.push_str(&format!(
                "{} = {} # {}\n",
                key.name, key.default, key.description
            ));
        }
    }

    body
}
