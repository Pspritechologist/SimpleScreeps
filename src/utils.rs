pub trait IterEnum {
	fn variants() -> &'static [Self] where Self: std::marker::Sized;
}
