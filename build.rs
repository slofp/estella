fn main() {
	println!("cargo:rustc-link-search=voicevox_core/c_api/lib");
	println!("cargo:rustc-link-lib=voicevox_core");
}