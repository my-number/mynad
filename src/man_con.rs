use pcsc::{Card, Error as PcscError};
use std::{
    mem,
    ops::DerefMut,
    sync::RwLock,
};

use crate::{error::Error, utils::open_card};
pub struct ManCon {
    card: RwLock<Option<Card>>,
}

impl Default for ManCon {
    fn default() -> Self {
        ManCon {
            card: Default::default(),
        }
    }
}

impl ManCon {
    pub fn connect(&self, name: String) -> Result<(), PcscError> {
        let card = open_card(name)?;
        let ptr = self.card.write();
        let mut wraped_ptr = ptr.unwrap();
        let mut_ptr = wraped_ptr.deref_mut();
        *mut_ptr = Some(card);
        Ok(())
    }
    pub fn disconnect(&self) -> Result<(), Error> {
        let ptr = self.card.write();
        let mut wrapped_ptr = ptr.unwrap();
        let mut_ptr = wrapped_ptr.deref_mut();
        let unwrapped_mut_ptr = mem::replace(mut_ptr, None);
        unwrapped_mut_ptr
            .unwrap()
            .disconnect(pcsc::Disposition::ResetCard)
            .map_err(|_| (Error::Execution("Unable to drop")))?;

        Ok(())
    }
    pub fn get_card(&self) -> &RwLock<Option<Card>> {
        return &self.card;
    }
}
