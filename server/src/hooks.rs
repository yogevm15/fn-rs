use crate::state;
use detour::RawDetour;
use pelite::pattern::Pattern;
use pelite::PeView;
use std::ops::Range;
use thiserror::Error;

pub struct Hook {
    target: Pattern,

    detour: *const (),

    is_needed_callback: Option<fn(state::State) -> bool>,
}

#[derive(Error, Debug)]
enum HookError {
    #[error("Pattern not found")]
    PatternNotFound,

    #[error("`{0}`")]
    DetourError(#[from] detour::Error),
}

impl Hook {
    pub fn apply(&self, view: &PeView, state: state::State) -> Result<(), HookError> {
        if self.is_needed_callback.is_some() && !self.is_needed_callback.unwrap()(state) {
            return Ok(());
        }

        let mut save = [0; 1];
        if view.scanner().finds_code(self.target.as_slice(), &mut save) {
            Err(HookError::PatternNotFound)?
        }

        let mut hook = unsafe { RawDetour::new(save[0] as *const (), self.detour) }?;
        unsafe { hook.enable() }?;
        Ok(())
    }
}
