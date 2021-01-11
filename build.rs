extern crate build_deps;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_deps::rerun_if_changed_paths("./assets/shaders/*").unwrap();

    let mut compiler = shaderc::Compiler::new().unwrap();
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let assets_path = format!("{}/{}", out_dir, "./assets/gen/shaders");

    // Create destination path if necessary
    std::fs::create_dir_all(&assets_path)?;

    for entry in std::fs::read_dir("./assets/shaders")? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let in_path = entry.path();
            let shader_type = in_path.extension().and_then(|ext| match ext.to_string_lossy().as_ref() {
                "vert" => Some(shaderc::ShaderKind::Vertex),
                "frag" => Some(shaderc::ShaderKind::Fragment),
                _ => None,
            });

            if let Some(shader_type) = shader_type {
                let source = std::fs::read_to_string(&in_path)?;
                let compiled_file = compiler.compile_into_spirv(&source, shader_type, entry.path().to_str().unwrap(), "main", None).unwrap();

                let compiled_bytes = compiled_file.as_binary_u8();

                // Determine the output path based on the input name
                let out_path = format!("{}/{}.spv", assets_path, in_path.file_name().unwrap().to_string_lossy().to_string());

                std::fs::write(&out_path, &compiled_bytes)?;
            }
        }
    }

    Ok(())
}
