fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use lobsterchat::component::{Colored, Component, NamedColor};
    use picolimbo_proto::{BytesMut, Encodeable, Identifier, Result};

    #[derive(Encodeable)]
    struct TestDerived {
        #[varint]
        some_number: i16,
        some_string: &'static str,
        some_id: Identifier,
        some_chat: Component,
    }

    #[test]
    fn test_derived_write() -> Result<()> {
        let mut buf = BytesMut::new();
        let value = TestDerived {
            some_number: 12345,
            some_string: "Hello, world!",
            some_id: "minecraft:diamond".into(),
            some_chat: Component::text("Some cool text")
                .color(NamedColor::Aqua)
                .bold(true),
        };
        value.encode(&mut buf)?;
        // dont worry, these numbers were generated
        assert_eq!(
            &[
                185, 96, 13, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 18, 109,
                105, 110, 101, 99, 114, 97, 102, 116, 58, 58, 100, 105, 97, 109, 111, 110, 100, 52,
                123, 34, 98, 111, 108, 100, 34, 58, 116, 114, 117, 101, 44, 34, 99, 111, 108, 111,
                114, 34, 58, 34, 97, 113, 117, 97, 34, 44, 34, 116, 101, 120, 116, 34, 58, 34, 83,
                111, 109, 101, 32, 99, 111, 111, 108, 32, 116, 101, 120, 116, 34, 125
            ],
            &buf[..]
        );
        Ok(())
    }
}
