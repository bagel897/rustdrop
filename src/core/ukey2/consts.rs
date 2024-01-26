use bytes::Bytes;
use hex_literal::hex;
pub const D2D_SALT: Bytes = Bytes::from_static(&hex!(
    "82AA55A0D397F88346CA1CEE8D3909B95F13FA7DEB1D4AB38376B8256DA85510"
));
pub const PT2_SALT: Bytes = Bytes::from_static(&hex!(
    "BF9D2A53C63616D75DB0A7165B91C1EF73E537F2427405FA23610A4BE657642E"
));
