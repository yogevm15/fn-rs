use crate::bindings::SpawnActorO;
use pelite::pattern;
use pelite::pattern::parse;
use pelite::pe64::Pe;
use pelite::pe64::{PeView, Rva, Va};
use std::ops::Range;
use std::process::exit;
use std::ptr::{null, null_mut};
use thiserror::Error;

pub struct State {}

const SPAWN_ACTOR_PATTERNS: [&str; 4] = ["40 53 56 57 48 83 EC 70 48 8B 05 ? ? ? ? 48 33 C4 48 89 44 24 ? 0F 28 1D ? ? ? ? 0F 57 D2 48 8B B4 24 ? ? ? ? 0F 28 CB", "40 53 48 83 EC 70 48 8B 05 ? ? ? ? 48 33 C4 48 89 44 24 ? 0F 28 1D ? ? ? ? 0F 57 D2 48 8B 9C 24 ? ? ? ? 0F 28 CB 0F 54 1D ? ? ? ? 0F 57", "48 89 5C 24 ? 55 56 57 48 8B EC 48 81 EC ? ? ? ? 48 8B 05 ? ? ? ? 48 33 C4 48 89 45 F0 0F 28 05 ? ? ? ? 48 8B FA 0F 28 0D ? ? ? ? 48 8B D9 48 8B 75 40 0F 29 45 C0 0F 28 05 ? ? ? ? 0F 29 45 E0 0F 29 4D D0 4D 85 C0 74 12 F3 41 0F 10 50 ? F2 41 0F 10 18", "48 89 5C 24 ? 55 56 57 48 8D 6C 24 ? 48 81 EC ? ? ? ? 48 8B 05 ? ? ? ? 48 33 C4 48 89 45 2F 0F 28 0D ? ? ? ? 48 8B FA 0F 28 15 ? ? ? ? 48 8B D9 0F"];

#[derive(Error, Debug)]
enum StateError {
    #[error("`{0}`")]
    ParseError(#[from] pattern::ParsePatError),

    #[error("Pattern not found")]
    PatternNotFound,

    #[error("`{0}`")]
    RvaError(pelite::Error),
}

impl State {
    pub fn new(view: &PeView) {
        let spawn_actor_o = Self::find_spawn_actor(view).unwrap().unwrap();
        println!("{}", spawn_actor_o as usize);
        let result =
            unsafe { spawn_actor_o(null_mut(), null_mut(), null_mut(), null_mut(), null_mut()) }
                as usize;
        exit(result as i32)
    }

    fn find_spawn_actor(view: &PeView) -> Result<SpawnActorO, StateError> {
        let address = &mut [0; 1];
        for pat in SPAWN_ACTOR_PATTERNS {
            if view
                .scanner()
                .finds(parse(pat)?.as_slice(), 0..u32::MAX, address)
            {
                return Ok(unsafe {
                    core::mem::transmute::<Va, SpawnActorO>(
                        view.rva_to_va(address[0]).map_err(StateError::RvaError)? as Va,
                    )
                });
            }
        }
        Err(StateError::PatternNotFound)
    }
}
