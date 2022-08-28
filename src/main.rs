use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use openapi_client_generator::{OpenAPI, GenerateLibraryOptions};
use openapi_client_generator::generate_library;
use openapi_client_generator::openapiv3::{SchemaKind, Type};
use openapi_client_generator::sourcegen::SourceGen;
use serde_yaml::Value;
use anyhow::Result;


fn modify_spec(spec: &mut OpenAPI) {
    spec.operations_mut().for_each(|(_, _, operation)| {
        if let Some(ref mut docs) = operation.external_docs {
            docs.url = format!("https://plaid.com/docs{}", docs.url);
        }
    });
    spec.schemas_mut().iter_mut().for_each(|(_, schema)| {
        let schema = schema.as_mut().unwrap();
        match &mut schema.schema_kind {
            SchemaKind::Type(Type::Object(ref mut o)) => {
                let props = &mut o.properties;
                props.shift_remove("client_id");
                props.shift_remove("secret")
            }
            _ => { return; }
        };
    });
    spec.schemas_mut().iter_mut().for_each(|(_, schema)| {
        let schema = schema.as_mut().unwrap();
        match &mut schema.schema_kind {
            // plaid has a bunch of AllOfs with additional fields that don't actually exist.
            SchemaKind::AllOf { ref mut all_of } => {
                all_of.retain(|ref_schema| {
                    let schema = match ref_schema.as_item() {
                        None => return true,
                        Some(schema) => schema,
                    };
                    match &schema.schema_kind {
                        SchemaKind::Type(Type::Object(_)) => false,
                        _ => true,
                    }
                })
            }
            _ => {}
        }
    });
    // spec.components.as_mut().unwrap().schemas.get_mut("DepositSwitchTokenCreateRequest").map(|schema| {
    //     let schema = schema.as_mut().unwrap();
    //     let props = schema.properties_mut().unwrap();
    //     props.shift_remove("client_id");
    //     props.shift_remove("secret");
    //     props.insert("target_access_token".to_string(), ReferenceOr::Item(Schema {
    //         schema_kind: SchemaKind::Type(Type::String(StringType {
    //             format: Default::default(),
    //             pattern: None,
    //             enumeration: vec![],
    //             min_length: None,
    //             max_length: None
    //         })),
    //         schema_data: SchemaData {
    //             nullable: false,
    //             read_only: false,
    //             write_only: false,
    //             deprecated: false,
    //             external_docs: None,
    //             example: None,
    //             title: None,
    //             description: None,
    //             discriminator: None,
    //             default: None,
    //             extensions: Default::default()
    //         },
    //     }));
    //     props.insert("target_account_id".to_string(), ReferenceOr::Item(Schema {
    //         schema_kind: SchemaKind::Type(Type::String(StringType {
    //             format: Default::default(),
    //             pattern: None,
    //             enumeration: vec![],
    //             min_length: None,
    //             max_length: None
    //         })),
    //         schema_data: SchemaData {
    //             nullable: false,
    //             read_only: false,
    //             write_only: false,
    //             deprecated: false,
    //             external_docs: None,
    //             example: None,
    //             title: None,
    //             description: None,
    //             discriminator: None,
    //             default: None,
    //             extensions: Default::default()
    //         },
    //     }));
    // }).unwrap();
}


fn main() -> Result<()> {
    let package_version = env::var("VERSION").expect("VERSION is not set.");
    let openapi_path = env::var("OPENAPI_PATH").expect("OPENAPI_PATH is not set.");
    let generator = SourceGen::from_str(&env::var("GENERATOR").expect("GENERATOR is not set."))?;
    let file = File::open(&openapi_path).expect("Could not open OpenAPI file.");
    let mut yaml: Value = serde_yaml::from_reader(file).expect("Could not parse OpenAPI file.");
    yaml["components"]["schemas"]["PartnerCustomersCreateRequest"]["type"] = Value::from("object");
    yaml["components"]["schemas"]["UserName"]["type"] = Value::from("object");
    let mut spec: OpenAPI = serde_yaml::from_value(yaml).expect("Could not structure OpenAPI file.");

    modify_spec(&mut spec);

    let repo_name = match generator {
        SourceGen::Rust => "plaid-rs",
        SourceGen::Python => "plaid-python",
        SourceGen::Typescript => "plaid-ts",
        SourceGen::Golang => "plaid-go",
    };
    let org_name = "libninjacom";
    let dest_path = PathBuf::from_str("..").unwrap().join(repo_name);
    let qualified_github_repo = format!("{}/{}", org_name, repo_name);
    let package_name = match generator {
        SourceGen::Python => "plaid2",
        _ => "plaid"
    }.to_string();

    generate_library(spec, GenerateLibraryOptions {
        package_name,
        service_name: "Plaid".to_string(),
        qualified_github_repo,
        dest_path,
        lib_rs_path: Some("template/src/lib.rs".into()),
        model_rs_path: Some("template/src/model.rs".into()),
        package_version,
        generator,
    }).unwrap();
    Ok(())
}
