// Development regression for the edge-cost diagnosis; not an INTEL verdict.
// Reuse the existing generic perceptual foundation, not World C's target law.
include!("intel2_unified_worlds.rs");

fn edge_cost_fact(evo: &mut EvoPhase, pre: &[f32], action: usize, post: &[f32]) {
    evo.observe_initial_real(pre, false);
    assert!(evo.observe_phase_native_rival_probe_result(action, post).is_some());
}

#[test]
fn exploration_edge_cost_does_not_starve_immediately_unknown_actions() {
    for familiar in 0..6usize {
        let (mut evo, l1) = foundation::build24();
        let current = foundation::scene(&l1, 0, 0);
        let unvisited_successor = foundation::scene(&l1, 1, 1);
        // A familiar one-step transition, but no outgoing successor experience.
        // Its successor frontier is maximal; its motor has the most support.
        for _ in 0..16 {
            edge_cost_fact(&mut evo, &current, familiar, &unvisited_successor);
        }
        evo.observe_initial_real(&current, false);
        let direct = evo.choose_phase_native_abstract_direct_action();
        assert!(direct.is_some() && direct != Some(familiar));
        let selected = evo.choose_phase_native_abstract_learned_drive_action();
        println!("EDGE_COST_LOCAL familiar={familiar} direct={direct:?} selected={selected:?} weights={:?}", evo.phase_native_drive_weights());
        assert!(selected.is_some(), "learned exploration must remain available");
        assert_ne!(selected, Some(familiar), "one-step frontier must not indefinitely displace immediately unknown experiments");
    }
}

#[test]
fn exploration_edge_cost_preserves_travel_to_remote_unknown_actions() {
    for exit_motor in 0..6usize {
        let (mut evo, l1) = foundation::build24();
        let current = foundation::scene(&l1, 2, 0);
        let remote = foundation::scene(&l1, 3, 1);
        // All local actions are known. Only one reaches a distinct frontier;
        // the others self-loop. The correction must not disable exploration
        // requiring travel, or simply replace it with direct-only sampling.
        for action in 0..6usize {
            for _ in 0..4 {
                let post = if action == exit_motor { &remote } else { &current };
                edge_cost_fact(&mut evo, &current, action, post);
            }
        }
        evo.observe_initial_real(&current, false);
        assert_eq!(evo.choose_phase_native_abstract_direct_action(), None);
        let selected = evo.choose_phase_native_abstract_learned_drive_action();
        println!("EDGE_COST_REMOTE exit={exit_motor} selected={selected:?}");
        assert_eq!(selected, Some(exit_motor), "known travel to unvisited frontier must remain usable");
    }
}
