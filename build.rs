use shaderc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=assets/shaders");

    let mut compiler = shaderc::Compiler::new().unwrap();

    // Create destination path if necessary
    std::fs::create_dir_all("assets/gen/shaders")?;

    for entry in std::fs::read_dir("assets/shaders")? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let in_path = entry.path();
            let shader_type = in_path.extension().and_then(|ext| {
                match ext.to_string_lossy().as_ref() {
                    "vert" => Some(shaderc::ShaderKind::Vertex),
                    "frag" => Some(shaderc::ShaderKind::Fragment),
                    _ => None,
                }
            });

            if let Some(shader_type) = shader_type {
                let source = std::fs::read_to_string(&in_path)?;
                let compiled_file = compiler.compile_into_spirv(&source, shader_type, entry.path().to_str().unwrap(), "main", None).unwrap();

                let compiled_bytes = compiled_file.as_binary_u8();

                // Determine the output path based on the input name
                let out_path = format!(
                    "assets/gen/shaders/{}.spv",
                    in_path.file_name().unwrap().to_string_lossy()
                );

                std::fs::write(&out_path, &compiled_bytes)?;
            }
        }
    }

    Ok(())
}