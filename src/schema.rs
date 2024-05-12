// https://cswr.github.io/JsonSchema/spec/grammar/

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TypedJsonSchema {
    #[serde(rename = "object")]
    Object {
        #[serde(rename = "additionalProperties")]
        additional_properties: bool,

        #[serde(
            rename = "patternProperties",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        pattern_properties: Option<HashMap<String, JsonSchema>>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        properties: Option<HashMap<String, JsonSchema>>,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        required: Option<Vec<String>>,
    },

    #[serde(rename = "array")]
    Array { items: Box<JsonSchema> },

    #[serde(rename = "boolean")]
    Boolean,

    #[serde(rename = "string")]
    String,

    #[serde(rename = "integer")]
    Integer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSchema {
    #[serde(flatten)]
    typed_schema: Option<TypedJsonSchema>,

    title: String,

    description: String,

    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    one_of: Option<Vec<JsonSchema>>,
}

impl JsonSchema {
    pub fn new(
        typed_schema: Option<TypedJsonSchema>,
        title: &str,
        description: &str,
        one_of: Option<Vec<JsonSchema>>,
    ) -> Self {
        Self {
            typed_schema,
            title: String::from(title),
            description: String::from(description),
            one_of,
        }
    }

    pub fn one_of(typed_schemas: Vec<TypedJsonSchema>, title: &str, description: &str) -> Self {
        let schemas = typed_schemas.into_iter().map(JsonSchema::from).collect();

        Self {
            typed_schema: None,
            title: title.to_string(),
            description: description.to_string(),
            one_of: Some(schemas),
        }
    }
}

impl From<TypedJsonSchema> for JsonSchema {
    fn from(value: TypedJsonSchema) -> Self {
        Self::new(Some(value), "", "", None)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonSchemaDocument {
    #[serde(rename = "$schema")]
    schema: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    definitions: Option<HashMap<String, JsonSchema>>,

    #[serde(flatten)]
    json_schema: JsonSchema,
}

pub fn to_writer<W: std::io::Write>(writer: W) -> serde_json::Result<()> {
    let json_schema_document = JsonSchemaDocument {
        id: None,
        schema: String::from("http://json-schema.org/draft-07/schema"),
        definitions: None,
        json_schema: JsonSchema::new(
            Some(TypedJsonSchema::Object {
                additional_properties: false,
                pattern_properties: None,
                properties: Some(HashMap::from([
                    (
                        String::from("config"),
                        JsonSchema::one_of(
                            vec![
                                TypedJsonSchema::Object {
                                    additional_properties: false,
                                    pattern_properties: None,
                                    properties: Some(HashMap::from([
                                        (
                                            "source_dir".to_string(),
                                            JsonSchema::new(
                                                Some(TypedJsonSchema::String),
                                                "Symlink source directory",
                                                "Prefix joined with each symlink target path",
                                                None,
                                            ),
                                        ),
                                        (
                                            "destination_dir".to_string(),
                                            JsonSchema::new(
                                                Some(TypedJsonSchema::String),
                                                "Symlink destination directory",
                                                "Prefix joined with each symlink name",
                                                None,
                                            ),
                                        ),
                                    ])),
                                    required: Some(vec![
                                        "source_dir".to_string(),
                                        "destination_dir".to_string(),
                                    ]),
                                },
                                TypedJsonSchema::Object {
                                    additional_properties: false,
                                    pattern_properties: None,
                                    properties: Some(HashMap::from([
                                        (
                                            "extends".to_string(),
                                            JsonSchema::new(
                                                Some(TypedJsonSchema::String),
                                                "Parent configuration",
                                                "Inherit default configuration values from another configuration file.",
                                                None,
                                            ),
                                        ),
                                        (
                                            "source_dir".to_string(),
                                            JsonSchema::new(
                                                Some(TypedJsonSchema::String),
                                                "Symlink source directory",
                                                "Prefix joined with each symlink target path",
                                                None,
                                            ),
                                        ),
                                        (
                                            "destination_dir".to_string(),
                                            JsonSchema::new(
                                                Some(TypedJsonSchema::String),
                                                "Symlink destination directory",
                                                "Prefix joined with each symlink name",
                                                None,
                                            ),
                                        ),
                                    ])),
                                    required: Some(vec!["extends".to_string()]),
                                },
                            ],
                            "Configuration",
                            "Either a root configuration or a child configuration that inherits default config values from another configuration file.",
                        ),
                    ),
                    (
                        String::from("symlinks"),
                        JsonSchema::new(
                            Some(TypedJsonSchema::Object {
                                additional_properties: false,
                                pattern_properties: Some(HashMap::from([(
                                    String::from(r"^[a-zA-Z0-9_./-]+$"),
                                    JsonSchema::new(
                                        Some(TypedJsonSchema::String),
                                        "Maps symlink name to target path relative to the configured source directory.",
                                        "",
                                        None,
                                    ),
                                )])),
                                properties: None,
                                required: None,
                            }),
                            "Symlinks",
                            "Object whose keys are symlink names and whose values are symlink target paths.",
                            None,
                        ),
                    ),
                    (
                        String::from("$schema"),
                        JsonSchema::new(
                            Some(TypedJsonSchema::String),
                            "JSON schema",
                            "URL or path to the JSON schema used to validate this document.",
                            None,
                        ),
                    ),
                ])),
                required: Some(vec!["config".to_string()]),
            }),
            "Symlonk configuration",
            "",
            None,
        ),
    };

    serde_json::to_writer(writer, &json_schema_document)
}
