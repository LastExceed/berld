pub mod io_extensions;
pub mod flagset;

//ideally this would be done with a #[derive()] macro instead
//but the boilerplate required for that is completely overkill for this use case
#[macro_export]
macro_rules! bulk_impl {
	($trait:ident for $($struct:ty),*) => { //todo: investigate if 'trait' can be restricted to :ty
		$(impl $trait for $struct {})*
	}
}