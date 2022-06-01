#[derive(Eq, PartialEq)]
struct X {
	foo: Option<i32>,
	bar: Option<i32>
}

macro_rules! bitfield_from_options_struct {
	($this:ident [$($field:ident),* $(,)?]) => {{
		let mut acc = 0u64;
		$(
			acc <<= 1;
			acc |= $this.$field.is_some() as u64;
		)*
		acc
	}}
}

pub fn foo() {
	let mut x = 1u64;
	println!("{:0b} - {:?}", x, x);
	x |= (true as u64) >> 1;
	println!("{:0b} - {:?}", x, x);

	let x = X {
		foo: Some(42),
		bar: Some(42)
	};

	let r = bitfield_from_options_struct!(x [foo, bar]);
	println!("{:0b}", r);
}