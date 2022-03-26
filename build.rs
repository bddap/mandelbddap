use std::{path, str::FromStr};

use spirv_builder::SpirvBuilder;
use spirv_cross::{glsl, spirv};

fn main() {
    let out_dir = path::PathBuf::from_str(&std::env::var("OUT_DIR").unwrap())
        .unwrap()
        .join("shaders");
    std::fs::create_dir_all(&out_dir).unwrap();

    println!("cargo:rerun-if-changed=./shaders");

    let shader_name = "sky-shader";

    // compile shader to spirv using https://github.com/EmbarkStudios/rust-gpu
    let module = SpirvBuilder::new(
        format!("./shaders/{shader_name}"),
        "spirv-unknown-vulkan1.1",
    )
    .build()
    .unwrap()
    .module;
    let spirv_module_path = module.unwrap_single();

    // transpile from spirv using https://github.com/grovesNL/spirv_cross
    let spirv_bin = std::fs::read(spirv_module_path).unwrap();
    let spirv_words = words_from_bytes(&spirv_bin);
    let spirv_module = spirv::Module::from_words(spirv_words);

    // glsl
    let mut ast = spirv::Ast::<glsl::Target>::parse(&spirv_module).unwrap();
    // place shaders in target dir
    for entry_point in ast.get_entry_points().unwrap() {
        let filename = sanitize_filename::sanitize(&entry_point.name);
        let filepath = out_dir.join(format!("{shader_name}-{filename}.glsl"));

        let mut options = glsl::CompilerOptions::default();
        //{
        // pub version: Version,
        // options.force_temporary = true;
        // options.vulkan_semantics = true;
        // options.separate_shader_objects = true;
        // pub flatten_multidimensional_arrays: bool,
        // options.enable_420_pack_extension = true;
        // options.emit_push_constant_as_uniform_buffer = true;
        // options.emit_uniform_buffer_as_plain_uniforms = true;
        // options.emit_line_directives = true;
        // pub enable_storage_image_qualifier_deduction: bool,
        // /// Whether to force all uninitialized variables to be initialized to zero.
        // pub force_zero_initialized_variables: bool,
        // pub vertex: CompilerVertexOptions,
        // pub fragment: CompilerFragmentOptions,
        // /// The name and execution model of the entry point to use. If no entry
        // /// point is specified, then the first entry point found will be used.
        // pub entry_point: Option<(String, spirv::ExecutionModel)>,
        //};
        options.version = glsl::Version::V4_60;
        options.entry_point = Some((entry_point.name, entry_point.execution_model));
        ast.set_compiler_options(&options).unwrap();

        // Compile to GLSL
        let shader = ast.compile().unwrap();

        std::fs::write(filepath, shader).unwrap();
    }

    // // hlsl
    // let mut ast = spirv::Ast::<hlsl::Target>::parse(&spirv_module).unwrap();
    // // place shaders in target dir
    // for entry_point in ast.get_entry_points().unwrap() {
    //     let filename = sanitize_filename::sanitize(&entry_point.name);
    //     let filepath = out_dir.join(format!("{shader_name}-{filename}.hlsl"));

    //     let mut options = hlsl::CompilerOptions::default();
    //     options.shader_model = hlsl::ShaderModel::V6_0;
    //     options.entry_point = Some((entry_point.name, entry_point.execution_model));
    //     ast.set_compiler_options(&options).unwrap();

    //     // Compile to GLSL
    //     let shader = ast.compile().unwrap();

    //     std::fs::write(filepath, shader).unwrap();
    // }
}

pub fn words_from_bytes(buf: &[u8]) -> &[u32] {
    assert!(buf.len() % std::mem::size_of::<u32>() == 0);
    // I don't know whether spriv_cross converts endianness.
    // If not, this build will like fail on some architectures.
    // Might need to convert endianness if that happens.
    unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr() as *const u32,
            buf.len() / std::mem::size_of::<u32>(),
        )
    }
}
