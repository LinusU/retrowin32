use win32_system::System;

#[win32_derive::dllexport]
pub fn InterlockedIncrement(sys: &dyn System, addend: Option<&mut u32>) -> u32 {
    let addend = addend.unwrap();
    *addend += 1;
    *addend
}

#[win32_derive::dllexport]
pub fn InterlockedDecrement(sys: &dyn System, addend: Option<&mut u32>) -> u32 {
    let addend = addend.unwrap();
    *addend -= 1;
    *addend
}

#[win32_derive::dllexport]
pub fn InterlockedCompareExchange(
    sys: &dyn System,
    destination: Option<&mut u32>,
    exchange: u32,
    comparand: u32,
) -> u32 {
    let destination = destination.unwrap();
    let original = *destination;
    if original == comparand {
        *destination = exchange;
    }
    original
}
