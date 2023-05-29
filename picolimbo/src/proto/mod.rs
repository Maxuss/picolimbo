use picolimbo_proto::Encodeable;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Packet {}

impl Encodeable for Packet {
    fn encode(&self, out: &mut picolimbo_proto::BytesMut) -> picolimbo_proto::Result<()> {
        Ok(())
    }
}
