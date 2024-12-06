use byteorder::ByteOrder;

pub trait MapBytes {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>, sample_size: usize) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder;
}

impl MapBytes for i16 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>, sample_size: usize) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        if sample_size == 2 {
            Some(B::read_i16(&[iter.next()?, iter.next()?]))
        } else {
            None
        }
    }
}

impl MapBytes for i32 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>, sample_size: usize) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        if sample_size == 4 {
            Some(B::read_i32(&[
                iter.next()?,
                iter.next()?,
                iter.next()?,
                iter.next()?,
            ]))
        } else {
            None
        }
    }
}

impl MapBytes for f32 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>, sample_size: usize) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        if sample_size == 3 {
            // read 24 bits
            let val = B::read_i24(&[iter.next()?, iter.next()?, iter.next()?]);

            // convert to f32
            Some(val as f32 / (1 << 23) as f32)
        } else if sample_size == 4 {
            Some(B::read_f32(&[
                iter.next()?,
                iter.next()?,
                iter.next()?,
                iter.next()?,
            ]))
        } else {
            None
        }
    }
}

impl MapBytes for u8 {
    fn map_bytes<B>(iter: &mut impl Iterator<Item = u8>, sample_size: usize) -> Option<Self>
    where
        Self: Sized,
        B: ByteOrder,
    {
        if sample_size == 1 {
            Some(iter.next()?)
        } else {
            None
        }
    }
}
