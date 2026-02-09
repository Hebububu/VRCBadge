fn main() {
    // Propagate ESP-IDF build environment (linker scripts, etc.)
    embuild::espidf::sysenv::output();

    // Compile Slint UI with resources embedded for software renderer
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);

    slint_build::compile_with_config("../ui/badge.slint", config).unwrap();
}
