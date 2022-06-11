pub fn username(name: String, discriminator: u16) -> String {
	format!("{}#{}", name, discriminator)
}
