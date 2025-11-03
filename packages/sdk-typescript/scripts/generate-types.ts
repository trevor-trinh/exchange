#!/usr/bin/env bun
import { mkdir, writeFile, readFile } from "node:fs/promises";
import { join } from "node:path";
import { compile } from "json-schema-to-typescript";

// Use import.meta.dir which is Bun-specific
const SCRIPT_DIR = import.meta.dir;
// Single schema file in packages/shared
const SCHEMA_FILE = join(SCRIPT_DIR, "../../../packages/shared/websocket.json");
// Output to SDK's src/types/generated
const OUTPUT_DIR = join(SCRIPT_DIR, "../src/types/generated");

async function main() {
  try {
    console.log("Schema file:", SCHEMA_FILE);
    console.log("Output dir:", OUTPUT_DIR);

    // Ensure output directory exists
    await mkdir(OUTPUT_DIR, { recursive: true });

    // Read and parse the schema
    const schemaContent = await readFile(SCHEMA_FILE, "utf-8");
    const schema = JSON.parse(schemaContent);

    // Create standalone schemas for each definition by including all definitions
    const typePromises = [];

    // Generate the main ClientMessage type
    typePromises.push(
      compile(schema, "ClientMessage", {
        additionalProperties: false,
        bannerComment: "",
      })
    );

    // Generate types for each definition with full schema context
    if (schema.definitions) {
      for (const [name, def] of Object.entries(schema.definitions)) {
        // Create a standalone schema for this definition that includes all definitions
        const standaloneSchema = {
          ...def,
          definitions: schema.definitions,
        };

        typePromises.push(
          compile(standaloneSchema as any, name, {
            additionalProperties: false,
            bannerComment: "",
          })
        );
      }
    }

    // Wait for all types to be generated
    const types = await Promise.all(typePromises);

    // Deduplicate types by tracking unique exports
    const uniqueTypes = new Set<string>();
    const deduplicatedTypes: string[] = [];

    for (const typeCode of types) {
      // Split by export statements
      const exports = typeCode.split(/(?=export )/g).filter(Boolean);

      for (const exportStatement of exports) {
        // Extract type signature to detect duplicates
        const match = exportStatement.match(/export (?:type|interface) (\w+)/);
        if (match) {
          const typeName = match[1];
          if (!uniqueTypes.has(typeName)) {
            uniqueTypes.add(typeName);
            deduplicatedTypes.push(exportStatement.trim());
          }
        }
      }
    }

    // Combine all unique types into one file
    const combinedTypes = deduplicatedTypes.join("\n\n") + "\n";

    // Write to websocket.ts
    const outputFile = join(OUTPUT_DIR, "websocket.ts");
    await writeFile(outputFile, combinedTypes);

    console.log(`✓ Generated websocket.ts with ${uniqueTypes.size} unique types`);
    console.log(`\n✅ Successfully generated TypeScript types in ${OUTPUT_DIR}`);
  } catch (error) {
    console.error("❌ Error generating types:", error);
    process.exit(1);
  }
}

main();
