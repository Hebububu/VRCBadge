fn main() {
    // Propagate ESP-IDF build environment (linker scripts, etc.)
    embuild::espidf::sysenv::output();

    // Explicit bitmap font sizes matching consolidated UI font sizes.
    // Bitmap fonts render faster and with better quality than SDF on embedded displays.
    std::env::set_var("SLINT_FONT_SIZES", "14,16,20,24,32");

    // Compile Slint UI with resources embedded for software renderer.
    // scale_factor(1.0) ensures images are pre-rendered at their declared pixel sizes
    // (e.g. 24x24 battery icons stay 24x24, not inflated to a larger default).
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer)
        .with_scale_factor(1.0);

    slint_build::compile_with_config("../ui/badge.slint", config).unwrap();
}
