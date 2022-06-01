extern crate glsl_to_spirv;

use std::error::Error;
use glsl_to_spirv::ShaderType;

fn main() -> Result<(), Box<dyn Error>> {
  // Tell the build script to only run again if we change our source shaders
  println!("cargo:rerun-if-changed=assets/shader/src");

  // Create destination path if necessary
  for entry in std::fs::read_dir("assets/shader/src")? {
    let entry = entry?;

    if entry.file_type()?.is_file() {
      let in_path = entry.path();

      // Support only vertex and fragment shaders currently
      let shader_type = in_path.extension().and_then(|ext| {
        match ext.to_string_lossy().as_ref() {
          "vert" => Some(ShaderType::Vertex),
          "frag" => Some(ShaderType::Fragment),
          _ => None,
        }
      });

      if let Some(shader_type) = shader_type {
        use std::io::Read;

        let source = std::fs::read_to_string(&in_path)?;
        let mut compiled_file = glsl_to_spirv::compile(&source, shader_type)?;
        let mut compiled_bytes = Vec::new();
        compiled_file.read_to_end(&mut compiled_bytes)?;

        // Determine the output path based on the input name
        let out_path = format!(
          "assets/shader/{}.spv",
          in_path.file_name().unwrap().to_string_lossy()
        );

        std::fs::write(&out_path, &compiled_bytes)?;
      }
    }
  }
  Ok(())
}