pub(crate) fn bytes_to_uint<T>(bytes: &[u8]) -> Option<T>
where
    T: num_traits::PrimInt
        + std::ops::AddAssign
        + std::ops::MulAssign
        + num_traits::FromPrimitive
        + num_traits::Unsigned,
{
    assert!(!bytes.is_empty());
    const OFF_SET: u8 = 48;
    let mut xten = T::one();
    let mut acc = T::zero();
    let ten = T::from_u16(10_u16).unwrap();
    for i in (0..bytes.len()).rev() {
        let b = bytes[i];
        if b > b'9' || b < b'0' {
            return None;
        }
        acc += T::from_u8(b - OFF_SET).unwrap() * xten;
        xten *= ten;
    }
    Some(acc)
}
