// mod hooks;
mod bindings;
mod state;

// use detour::RawDetour;
use ctor::ctor;
use pelite::pe64::PeView;
// const HOOKS: [hooks::Hook; 0] = [];

#[ctor]
fn init() {
    let curr_view = unsafe { PeView::new() };
    state::State::new(&curr_view);
    // for hook in HOOKS {
    //     hook.apply(&curr_view, state).unwrap();
    // }
}
