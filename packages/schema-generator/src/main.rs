use backend::models::api::{ClientMessage, ServerMessage};
use schemars::schema_for;
use std::{fs, path::Path};

fn main() {
    // Get the workspace root (2 levels up from this crate)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = Path::new(manifest_dir).parent().unwrap().parent().unwrap();
    let output_dir = workspace_root.join("packages/shared");

    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Generate schemas for WebSocket types
    let client_schema = schema_for!(ClientMessage);
    let server_schema = schema_for!(ServerMessage);

    // Combine both schemas into a single file with all definitions
    let mut combined = client_schema.clone();

    // Merge all definitions from server schema
    for (key, value) in server_schema.definitions {
        combined.definitions.insert(key, value);
    }

    // Add ServerMessage as a top-level definition
    combined.definitions.insert("ServerMessage".to_string(), server_schema.schema.into());

    // Write combined schema to websocket.json
    let file_path = output_dir.join("websocket.json");
    fs::write(
        &file_path,
        serde_json::to_string_pretty(&combined).expect("Failed to serialize schema"),
    )
    .expect("Failed to write schema file");

    println!("Generated schema: {}", file_path.display());
    println!("\n✅ Successfully generated WebSocket schema!");
}
