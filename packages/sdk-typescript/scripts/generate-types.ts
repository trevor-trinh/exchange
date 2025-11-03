#!/usr/bin/env bun
import { mkdir, writeFile, readdir } from "node:fs/promises";
import { join, dirname } from "node:path";
import { compileFromFile } from "json-schema-to-typescript";

// Use import.meta.dir which is Bun-specific
const SCRIPT_DIR = import.meta.dir;
// Schema files are in the workspace root
const SCHEMA_DIR = join(SCRIPT_DIR, "../../../packages/shared/schema");
// Output to SDK's src/types/generated
const OUTPUT_DIR = join(SCRIPT_DIR, "../src/types/generated");

async function main() {
  try {
    console.log("Schema dir:", SCHEMA_DIR);
    console.log("Output dir:", OUTPUT_DIR);

    // Ensure output directory exists
    await mkdir(OUTPUT_DIR, { recursive: true });

    // Get all JSON files from the schema directory
    const files = await readdir(SCHEMA_DIR);
    const schemaFiles = files.filter((file) => file.endsWith(".json")).map((file) => join(SCHEMA_DIR, file));

    console.log(`Found ${schemaFiles.length} schema files to process...`);

    // Generate TypeScript types and combine into single file, de-duplicating shared types
    const typeMap = new Map<string, string>();

    for (const schemaFile of schemaFiles.sort()) {
      const fileName = schemaFile.split("/").pop()?.replace(".json", "");
      const ts = await compileFromFile(schemaFile, {
        additionalProperties: false,
        bannerComment: "",
      });

      // Split by export statements and collect unique types
      const exports = ts.split(/(?=export )/g).filter(Boolean);
      for (const exportStatement of exports) {
        // Extract type name from export
        const match = exportStatement.match(/export (?:type|interface) (\w+)/);
        if (match) {
          const typeName = match[1];
          if (!typeMap.has(typeName)) {
            typeMap.set(typeName, exportStatement.trim());
          }
        }
      }
    }

    // Combine all unique types
    const combinedTypes = Array.from(typeMap.values()).join("\n\n") + "\n";

    // Write combined types to websocket.ts
    const outputFile = join(OUTPUT_DIR, "websocket.ts");
    await writeFile(outputFile, combinedTypes);
    console.log(`✓ Generated websocket.ts (${typeMap.size} unique types)`);

    console.log(`\n✅ Successfully generated TypeScript types in ${OUTPUT_DIR}`);
  } catch (error) {
    console.error("❌ Error generating types:", error);
    process.exit(1);
  }
}

main();
