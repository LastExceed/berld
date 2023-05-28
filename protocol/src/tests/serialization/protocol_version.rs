use crate::generate_serialization_tests;

generate_serialization_tests!(
	ProtocolVersion(0x12345678),
	[0x78, 0x56, 0x34, 0x12]
);