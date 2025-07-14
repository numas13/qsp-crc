use qsp::{Crc, Packed, crc32, crc64};

#[derive(Copy, Clone, Packed)]
#[repr(C, packed)]
struct Foo {
    x: u8,
    y: u32,
    z: u8,
}

#[derive(Crc)]
#[repr(C)]
struct Bar<T> {
    f: T,
    x: u8,
    y: u32,
}

#[derive(Crc)]
struct Baz<'a> {
    x: &'a u8,
    y: &'a u32,
    z: &'a u8,
}

#[rustfmt::skip]
fn main() {
    let f = Foo { x: 1, y: 2, z: 3 };

    dbg!(crc32(&Foo { x: 1, y: 2, z: 3 }));
    dbg!(crc32(&Baz { x: &1, y: &2, z: &3 }));
    dbg!(crc32(&[1_u8, 2, 0, 0, 0, 3]));
    dbg!(crc32(&Bar { f, x: 1, y: 2 }));
    dbg!(crc32(&[1_u8, 2, 0, 0, 0, 3, 1, 2, 0, 0, 0]));

    dbg!(crc64(&Foo { x: 1, y: 2, z: 3 }));
    dbg!(crc64(&Baz { x: &1, y: &2, z: &3 }));
    dbg!(crc64(&[1_u8, 2, 0, 0, 0, 3]));
    dbg!(crc64(&Bar { f, x: 1, y: 2 }));
    dbg!(crc64(&[1_u8, 2, 0, 0, 0, 3, 1, 2, 0, 0, 0]));
}
