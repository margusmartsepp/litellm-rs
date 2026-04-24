use std::env;
use std::fs;
use std::path::Path;
use typify::{TypeSpace, TypeSpaceSettings};
use regex::Regex;

fn main() {
    println!("cargo:rerun-if-changed=specs/");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let providers = vec!["openai", "anthropic"];

    for provider in providers {
        let json_path = format!("specs/{}.json", provider);
        let yaml_path = format!("specs/{}.yaml", provider);

        let spec_path = if Path::new(&json_path).exists() {
            json_path
        } else if Path::new(&yaml_path).exists() {
            yaml_path
        } else {
            continue;
        };

        let dest_path = Path::new(&out_dir).join(format!("{}_types.rs", provider));
        let mut content = fs::read_to_string(&spec_path).expect("Read error");

        // 1. Surgical String Taming
        content = tames_overflow_numbers(&content);
        let mut schema_value: serde_json::Value = if spec_path.ends_with(".yaml") {
            serde_yaml::from_str(&content).expect("YAML parse error")
        } else {
            serde_json::from_str(&content).expect("JSON parse error")
        };

        // 2. Strip extensions and fix drafts compatibility
        fix_openapi_schema(&mut schema_value);

        // Fix ModelIdsShared flattening issue in definitions or components/schemas
        for key in ["definitions", "components"] {
            if let Some(container) = schema_value.get_mut(key) {
                let target = if key == "components" {
                    container.get_mut("schemas")
                } else {
                    Some(container)
                };

                if let Some(schemas) = target {
                    if let Some(defs_map) = schemas.as_object_mut() {
                        if let Some(model_def) = defs_map.get_mut("ModelIdsShared") {
                            if let Some(model_obj) = model_def.as_object_mut() {
                                model_obj.clear();
                                model_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                            }
                        }
                        // Fix Model struct optionality for LM Studio compatibility
                        if let Some(model_def) = defs_map.get_mut("Model") {
                            if let Some(model_obj) = model_def.as_object_mut() {
                                if let Some(serde_json::Value::Array(required)) = model_obj.get_mut("required") {
                                    required.retain(|v| v.as_str() != Some("created"));
                                }
                                if let Some(props) = model_obj.get_mut("properties") {
                                    if let Some(created) = props.get_mut("created") {
                                        if let Some(created_obj) = created.as_object_mut() {
                                            created_obj.insert("nullable".to_string(), serde_json::json!(true));
                                        }
                                    }
                                }
                            }
                        }
                        // Make problematic fields optional and nullable across all response-related definitions
                        let problematic_fields = ["finish_reason", "logprobs", "tool_calls", "created"];
                        for (name, def) in defs_map.iter_mut() {
                            if name.contains("Response") || name.contains("Choice") || name.contains("Delta") || name.contains("Usage") {
                                if let Some(obj) = def.as_object_mut() {
                                    // 1. Remove from required
                                    if let Some(serde_json::Value::Array(required)) = obj.get_mut("required") {
                                        required.retain(|v| !problematic_fields.contains(&v.as_str().unwrap_or("")));
                                    }
                                    // 2. Make nullable in properties
                                    if let Some(props) = obj.get_mut("properties") {
                                        if let Some(props_map) = props.as_object_mut() {
                                            for field in problematic_fields {
                                                if let Some(prop) = props_map.get_mut(field) {
                                                    if let Some(prop_obj) = prop.as_object_mut() {
                                                        prop_obj.insert("nullable".to_string(), serde_json::json!(true));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    // 3. Handle nested anyOf/allOf/oneOf
                                    for key in ["allOf", "anyOf", "oneOf"] {
                                        if let Some(serde_json::Value::Array(arr)) = obj.get_mut(key) {
                                            for item in arr {
                                                if let Some(item_obj) = item.as_object_mut() {
                                                    if let Some(serde_json::Value::Array(req)) = item_obj.get_mut("required") {
                                                        req.retain(|v| !problematic_fields.contains(&v.as_str().unwrap_or("")));
                                                    }
                                                    if let Some(props) = item_obj.get_mut("properties") {
                                                        if let Some(props_map) = props.as_object_mut() {
                                                            for field in problematic_fields {
                                                                if let Some(prop) = props_map.get_mut(field) {
                                                                    if let Some(prop_obj) = prop.as_object_mut() {
                                                                        prop_obj.insert("nullable".to_string(), serde_json::json!(true));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Typify 0.6.1 Configuration
        let settings = TypeSpaceSettings::default();
        // We avoid forcing 'Default' derivation globally because it causes E0665 on enums
        // without default variants. Typify will still derive it where appropriate.

        let mut type_space = TypeSpace::new(&settings);

        // Inject schemas from components into the space.
        // We add all definitions at once and use a dummy property map to force
        // generation of all component types. This avoids duplicate generation
        // and naming conflicts seen with individual additions.
        if let Some(schemas) = schema_value.get("components").and_then(|c| c.get("schemas")) {
            let schemas_obj = schemas.as_object().unwrap();

            println!("cargo:warning=Processing provider schemas: {}", provider);

            let mut properties = serde_json::Map::new();
            for (name, _) in schemas_obj {
                println!("cargo:warning=  - Schema: {}", name);
                if name == "AddUploadPartRequest" { continue; }
                properties.insert(name.clone(), serde_json::json!({
                    "$ref": format!("#/definitions/{}", name)
                }));
            }

            let wrapper = serde_json::json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "definitions": schemas_obj,
                "type": "object",
                "properties": properties
            });

            match serde_json::from_value::<schemars::schema::RootSchema>(wrapper) {
                Ok(root_schema) => {
                    type_space.add_root_schema(root_schema).expect("Failed to add root schema");
                }
                Err(e) => {
                    panic!("JSON Schema conversion failed for {}: {}", provider, e);
                }
            }
        }

        let rendered = type_space.to_stream().to_string();
        fs::write(dest_path, rendered).expect("Write error");
    }
}

fn tames_overflow_numbers(input: &str) -> String {
    let re = Regex::new(r"(minimum|maximum):\s*-?\d{19,}").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let key = caps.get(1).unwrap().as_str();
        if key == "minimum" {
            format!("{}: -9223372036854775808", key)
        } else {
            format!("{}: 9223372036854775807", key)
        }
    }).into_owned()
}

fn fix_openapi_schema(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            map.retain(|k, _| !k.starts_with("x-"));

            // Remove "default" keys which often confuse typify's validation
            map.remove("default");

            // If this object has a 'required' list, remove any property that we've determined is nullable
            if let Some(serde_json::Value::Array(required)) = map.get("required") {
                let mut to_remove = Vec::new();
                if let Some(props) = map.get("properties") {
                    if let Some(props_map) = props.as_object() {
                        for name_val in required {
                            if let Some(name) = name_val.as_str() {
                                if let Some(prop) = props_map.get(name) {
                                    if let Some(obj) = prop.as_object() {
                                        for key in ["anyOf", "oneOf"] {
                                            if let Some(serde_json::Value::Array(options)) = obj.get(key) {
                                                if options.iter().any(|v| v.get("type") == Some(&serde_json::json!("null"))) {
                                                    to_remove.push(name.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !to_remove.is_empty() {
                    let required_mut = map.get_mut("required").unwrap().as_array_mut().unwrap();
                    required_mut.retain(|v| !to_remove.contains(&v.as_str().unwrap().to_string()));
                }
            }

            // 1. Flatten anyOf/oneOf with null to avoid duplicate naming in typify 0.6.1.
            // This prevents the generation of recursive newtype wrappers like
            // struct T(Option<T>) which cause E0428 and other compilation errors.
            for key in ["anyOf", "oneOf"] {
                if let Some(serde_json::Value::Array(options)) = map.get(key) {
                    if options.len() == 2 {
                        let null_idx = options.iter().position(|v| v.get("type") == Some(&serde_json::json!("null")));
                        if let Some(idx) = null_idx {
                            let non_null = options[1 - idx].clone();
                            map.remove(key);
                            if let serde_json::Value::Object(inner) = non_null {
                                for (k, v) in inner {
                                    map.insert(k, v);
                                }
                                map.insert("nullable".to_string(), serde_json::json!(true));
                            } else {
                                // If it's not an object (e.g., a $ref), create an object with $ref and nullable
                                let mut new_obj = serde_json::Map::new();
                                if let Some(ref_str) = non_null.as_str() {
                                     new_obj.insert("$ref".to_string(), serde_json::Value::String(ref_str.to_string()));
                                } else if let Some(obj) = non_null.as_object() {
                                     for (k, v) in obj {
                                         new_obj.insert(k.clone(), v.clone());
                                     }
                                }
                                new_obj.insert("nullable".to_string(), serde_json::json!(true));
                                *val = serde_json::Value::Object(new_obj);
                                return; // Done with this node
                            }
                        }
                    }
                }
            }

            // Remove constraints that cause typify to panic during to_stream()
            map.remove("maxLength");
            map.remove("minLength");
            map.remove("pattern");
            map.remove("format");
            map.remove("not");
            map.remove("title");

            // Specific fix for 'model' properties that cause problematic flattening in typify
            if let Some(props) = map.get_mut("properties") {
                if let Some(props_map) = props.as_object_mut() {
                    if let Some(model_prop) = props_map.get_mut("model") {
                        if let Some(model_obj) = model_prop.as_object_mut() {
                            if model_obj.contains_key("anyOf") || model_obj.contains_key("oneOf") {
                                model_obj.clear();
                                model_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                            }
                        }
                    }
                }
            }

            // Recurse into allOf/anyOf/oneOf arrays
            for key in ["allOf", "anyOf", "oneOf"] {
                if let Some(serde_json::Value::Array(arr)) = map.get_mut(key) {
                    for item in arr {
                        fix_openapi_schema(item);
                    }
                }
            }

            // Recurse into properties
            if let Some(serde_json::Value::Object(props)) = map.get_mut("properties") {
                for (_, prop) in props {
                    fix_openapi_schema(prop);
                }
            }

            // Recurse into items (for arrays)
            if let Some(items) = map.get_mut("items") {
                fix_openapi_schema(items);
            }

            // Fix typify panic on string enums without "type": "string"
            if let Some(serde_json::Value::Array(enum_arr)) = map.get("enum") {
                if enum_arr.len() == 1 {
                    let single_val = enum_arr[0].clone();
                    map.remove("enum");
                    map.insert("const".to_string(), single_val);
                } else if !map.contains_key("type") && enum_arr.iter().all(|v| v.is_string()) {
                    map.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                }
            }

            // If $ref is present, remove other structural fields to prevent typify panic
            if map.contains_key("$ref") {
                let allowed_keys = ["$ref", "description", "summary", "nullable"];
                map.retain(|k, _| allowed_keys.contains(&k.as_str()));
            }

            // Rewrite $ref
            if let Some(serde_json::Value::String(ref_str)) = map.get_mut("$ref") {
                if ref_str.starts_with("#/components/schemas/") {
                    *ref_str = ref_str.replace("#/components/schemas/", "#/definitions/");
                }
            }

            // Fix Draft 04 to Draft 07 compatibility for exclusiveMinimum/Maximum
            if let Some(serde_json::Value::Bool(true)) = map.get("exclusiveMinimum") {
                if let Some(min_val) = map.remove("minimum") {
                    map.insert("exclusiveMinimum".to_string(), min_val);
                } else {
                    map.remove("exclusiveMinimum");
                }
            } else if let Some(serde_json::Value::Bool(_)) = map.get("exclusiveMinimum") {
                map.remove("exclusiveMinimum");
            }

            if let Some(serde_json::Value::Bool(true)) = map.get("exclusiveMaximum") {
                if let Some(max_val) = map.remove("maximum") {
                    map.insert("exclusiveMaximum".to_string(), max_val);
                } else {
                    map.remove("exclusiveMaximum");
                }
            } else if let Some(serde_json::Value::Bool(_)) = map.get("exclusiveMaximum") {
                map.remove("exclusiveMaximum");
            }

            for v in map.values_mut() {
                fix_openapi_schema(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                fix_openapi_schema(v);
            }
        }
        _ => {}
    }
}
