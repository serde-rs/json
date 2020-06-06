//! Big integer type definition.

use super::math::*;
use crate::lib::Vec;

/// Storage for a big integer type.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Bigint {
    /// Internal storage for the Bigint, in little-endian order.
    pub(crate) data: Vec<Limb>,
}

impl Default for Bigint {
    fn default() -> Self {
        // We want to repeated reallocations at smaller volumes.
        let mut bigint = Bigint {
            data: Vec::<Limb>::default(),
        };
        reserve(&mut bigint.data, 20);
        bigint
    }
}

impl Math for Bigint {
    #[inline]
    fn data<'a>(&'a self) -> &'a Vec<Limb> {
        &self.data
    }

    #[inline]
    fn data_mut<'a>(&'a mut self) -> &'a mut Vec<Limb> {
        &mut self.data
    }
}
