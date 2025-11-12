use backend::models::api::{ClientMessage, ServerMessage};
use schemars::{JsonSchema, SchemaGenerator};
use std::{fs, path::Path};

// Wrapper type to include both message types in the schema
#[derive(JsonSchema)]
#[allow(dead_code)]
enum WebSocketMessages {
    Client(ClientMessage),
    Server(ServerMessage),
}

fn main() {
    // Get the workspace root (3 levels up from this binary: bin -> src -> backend -> apps -> workspace)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    let output_dir = workspace_root.join("packages/shared");

    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Create a schema generator and generate schemas for both types
    let mut generator = SchemaGenerator::default();

    // Generate subschemas for both message types to populate definitions
    generator.subschema_for::<ClientMessage>();
    generator.subschema_for::<ServerMessage>();

    // Create root schema with all definitions
    let root_schema = generator.into_root_schema_for::<WebSocketMessages>();

    // Write combined schema to websocket.json
    let file_path = output_dir.join("websocket.json");
    fs::write(
        &file_path,
        serde_json::to_string_pretty(&root_schema).expect("Failed to serialize schema"),
    )
    .expect("Failed to write schema file");

    println!("Generated schema: {}", file_path.display());
    println!("\n✅ Successfully generated WebSocket schema!");
}
