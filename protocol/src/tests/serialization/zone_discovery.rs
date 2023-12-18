use crate::{generate_serialization_tests, packet::area_request::Zone};

generate_serialization_tests!(
	AreaRequest::<Zone>([0x12345678, 0x11223344].into()),
	[0x78, 0x56, 0x34, 0x12, 0x44, 0x33, 0x22, 0x11]
);