fn main() {
    // Propagate ESP-IDF build environment (linker scripts, etc.)
    embuild::espidf::sysenv::output();

    // Compile Slint UI with resources embedded for software renderer.
    // scale_factor(1.0) ensures images are pre-rendered at their declared pixel sizes
    // (e.g. 24x24 battery icons stay 24x24, not inflated to a larger default).
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer)
        .with_scale_factor(1.0)
        .with_sdf_fonts(true);

    slint_build::compile_with_config("../ui/badge.slint", config).unwrap();
}
