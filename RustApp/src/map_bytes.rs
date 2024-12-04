use byteordered::byteorder::ByteOrder;

pub trait MapBytes {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder;
}

impl MapBytes for i16 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        Some(B::read_i16(&[iter.next()?, iter.next()?]))
    }
}

impl MapBytes for i32 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        Some(B::read_i32(&[
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
        ]))
    }
}

impl MapBytes for f32 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        Some(B::read_f32(&[
            iter.next()?,
            iter.next()?,
            iter.next()?,
            iter.next()?,
        ]))
    }
}
