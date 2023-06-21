
use super::game::{GSEvent, GameState, Accumulatee};

pub fn repeat_periodic(spawner: impl Accumulatee, count: usize, offset: f32, spacing: f32) -> Vec<GSEvent> {
    let mut out = vec![];
    for i in 0..count {
        let cloned = spawner.box_clone();
        out.push(GSEvent(
            i as f32 * spacing + offset,
            cloned
        ));
    }
    out
}

/// Takes a list of events, clones and offsets them for repetition of events
pub fn clone_offset(events: &Vec<GSEvent>, offset: f32) -> Vec<GSEvent> {
    let mut new_ev: Vec<GSEvent> = Vec::with_capacity(events.len());
    for i in events {
        new_ev.push(GSEvent(i.0 + offset, i.1.box_clone()));
    }
    new_ev
}

pub fn offset(mut events: Vec<GSEvent>, offset: f32) -> Vec<GSEvent> {
    events.iter_mut().map(|e|e.0 += offset).for_each(drop);
    events
}

/// Removes events within a certain time range
pub fn remove(events: Vec<GSEvent>, from: f32, to: f32) -> Vec<GSEvent> {
    events.into_iter().filter(|e|!(from..=to).contains(&e.0)).collect()
}
