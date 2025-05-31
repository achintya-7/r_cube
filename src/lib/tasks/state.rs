use std::collections::HashMap;

use super::types::State;

pub fn valid_state_transition(src: &State, dst: &State) -> bool {
    let state_transition_map: HashMap<State, Vec<State>> = {
        let mut map = HashMap::new();
        map.insert(State::Pending, vec![State::Scheduled]);
        map.insert(
            State::Scheduled,
            vec![State::Scheduled, State::Running, State::Failed],
        );
        map.insert(
            State::Running,
            vec![State::Running, State::Completed, State::Failed],
        );
        map.insert(State::Completed, vec![]);
        map.insert(State::Failed, vec![]);
        map
    };

    if let Some(valid_states) = state_transition_map.get(&src) {
        valid_states.contains(&dst)
    } else {
        false
    }
}
