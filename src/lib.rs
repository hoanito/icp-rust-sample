use candid::CandidType;
use ic_cdk_macros::*;
use serde::Deserialize;
use std::cell::RefCell;
mod env;

use crate::env::{CanisterEnvironment, EmptyEnvironment, Environment};

type TimeMillis = u64;
//create global state/data
// IC only single threaded. First time any thread access this global variable it will init the default value to that thread
thread_local! {
    static RUNTIME_STATE: RefCell<RuntimeState> = RefCell::default();
}

//#[derive(Default)] //pass the default value to thread
struct RuntimeState {
    env: Box<dyn Environment>,
    data: Achievement,
}

// this replace #[derive(Default)] because Environment is a trait, cant use the default impl
impl Default for RuntimeState {
    fn default() -> Self {
        RuntimeState {
            env: Box::new(EmptyEnvironment {}),
            data: Achievement::default(),
        }
    }
}

#[derive(Debug, Default, CandidType, Deserialize)] // Need to derive default as Achievement exist in RuntimeState
                                                   // Need Candid type and Deserialize to restore data after upgrade
struct Achievement {
    items: Vec<AchievementItem>,
}

#[derive(Debug, CandidType, Clone, Deserialize)] //This will deserialize to byte when candid return value to callers
                                                 // Need Deserialize to deserialize Achievement
struct AchievementItem {
    id: u32,
    name: String,
    done: bool,
    date_added: TimeMillis,
}

#[derive(CandidType, Deserialize)]
struct CanisterInfo {
    balance: u64,
    caller: String,
}

#[query]
fn get_canister_info() -> CanisterInfo {
    CanisterInfo {
        balance: ic_cdk::api::canister_balance(),
        caller: ic_cdk::api::caller().to_text(),
    }
}

#[update] //publicly exposed API
fn add(name: String) -> u32 {
    //access to global state
    RUNTIME_STATE.with(|state| add_impl(name, &mut state.borrow_mut())) //state is only accessible within .with() using borrow()
}

// split to separate function to test easier
fn add_impl(name: String, runtime_state: &mut RuntimeState) -> u32 {
    let id = (runtime_state.data.items.len() as u32) + 2;

    let current_time = runtime_state.env.now();

    runtime_state.data.items.push(AchievementItem {
        id,
        name,
        done: false,
        date_added: current_time,
    });

    id
}

#[query]
fn get() -> Vec<AchievementItem> {
    RUNTIME_STATE.with(|state| get_impl(&state.borrow()))
}

fn get_impl(runtime_state: &RuntimeState) -> Vec<AchievementItem> {
    runtime_state.data.items.clone() // return a clone value, not reference
}

#[query]
fn mark_done(id: u32) -> bool {
    RUNTIME_STATE.with(|state| mark_done_impl(id, &mut state.borrow_mut()))
}

fn mark_done_impl(id: u32, runtime_state: &mut RuntimeState) -> bool {
    let length = runtime_state.data.items.len() as u32;
    if id >= length {
        false
    } else {
        runtime_state.data.items[id as usize].done = true;
        true
    }
}

//---------------------
// Init Canister
//---------------------
#[init]
fn init() {
    RUNTIME_STATE.with(|state| {
        *state.borrow_mut() = RuntimeState {
            env: Box::new(CanisterEnvironment::new()),
            data: Achievement::default(),
        }
    })
}
//---------------------
// Init Canister
//---------------------

//---------------------
// Data Persistence for Canister upgrade
//---------------------

// Save data before update
#[pre_upgrade]
fn pre_upgrade() {
    //stable_save need a tuple (state.borrow(),) as param
    RUNTIME_STATE.with(|state| ic_cdk::storage::stable_save((&state.borrow().data,)).unwrap())
}

#[post_upgrade]
fn post_upgrade() {
    //NOTE: if there is no data in  state, this will fail. Comment out and run if there is no data

    //restore data from stable memory
    let (saved_achievement_data,): (Achievement,) = ic_cdk::storage::stable_restore().unwrap(); //return tuple

    let runtime_state = RuntimeState {
        env: Box::new(CanisterEnvironment::new()),
        data: saved_achievement_data,
    };

    RUNTIME_STATE.with(|state| *state.borrow_mut() = runtime_state)
}
//---------------------
// Data Persistence for Canister upgrade
//---------------------

//---------------------
// Test: at compile time, test only run when configuration set to test
//---------------------
#[cfg(test)]

mod test {
    use super::*;
    use crate::env::TestEnvironment;
    #[test]
    fn add_then_get() {
        let mut runtime_state = RuntimeState {
            env: Box::new(TestEnvironment { now: 1 }),
            data: Achievement::default(),
        };

        let new_achievement_name = String::from("Buy groceries");

        let new_achievement_id = add_impl(new_achievement_name.clone(), &mut runtime_state);

        let new_achievement_list = get_impl(&runtime_state);

        assert_eq!(new_achievement_list.len(), 1);

        assert_eq!(
            new_achievement_list.first().unwrap().name,
            new_achievement_name
        );

        assert_eq!(new_achievement_list.first().unwrap().date_added, 1);

        let new_achievement_done = mark_done(new_achievement_id);

        assert_eq!(new_achievement_list.first().unwrap().done, false);
    }
}
