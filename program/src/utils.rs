use crate::{
    state::{PREFIX, DELEGATE, bump_delegate},
};

pub fn get_seeds_delegate() -> [&'static [u8]; 3] {
    [PREFIX.as_bytes(), DELEGATE.as_bytes(), &[bump_delegate]]
}