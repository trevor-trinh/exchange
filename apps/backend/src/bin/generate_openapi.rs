use std::fs;
use utoipa::OpenApi;

use backend::api::rest::ApiDoc;

fn main() {
    println!("Generating OpenAPI specification...");

    // Generate the OpenAPI spec
    let openapi_spec = ApiDoc::openapi();
    let openapi_json =
        serde_json::to_string_pretty(&openapi_spec).expect("Failed to serialize OpenAPI spec");

    // Write directly to the shared package
    let output_path = "../../packages/shared/openapi.json";
    if let Err(e) = fs::write(output_path, &openapi_json) {
        eprintln!("Error: Could not write {}: {}", output_path, e);
        std::process::exit(1);
    }

    println!("✅ Generated {}", output_path);
    println!("📄 {} bytes written", openapi_json.len());
}
