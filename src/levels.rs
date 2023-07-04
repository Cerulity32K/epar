use std::f32::{consts::{PI, TAU, FRAC_PI_2}, NEG_INFINITY};

use macroquad::{prelude::{vec2, ORANGE, BLACK, WHITE, Vec2, RED, YELLOW, SKYBLUE, GRAY}, window::{screen_width, screen_height}, rand::gen_range};

use crate::{game::{GameState, GSEvent, UpdateAccumulator, ModifyArgs}, generators::{repeat_periodic, clone_offset, remove}, spawners::{HorLaserSpawner, LaserSpawner, BombSideSpawner}, game_objects::{Obst, Pellet, Periodic, SlamLaser, RotatableRect, Bomb, RotatingRect, SmokeProj, SmokeEvent, Obstacle, GOLGrid, GrowLaser}, utils::{cmul, gay, mix, screen_center, screen_size, rand_vec, floor_vec, screen, tev_rep, ez, repeat_events, rep_off}};

/// Function OBstacle Event
macro_rules! fobe {
    ($time:expr, $obsts:expr) => {
        GSEvent($time, Box::new(|gs, ()| for i in $obsts {
            gs.add_obstacle(Obst::new(Box::new(i())))
        } ))
    };
}

/// F-777 - Inferno (Pyro's Theme)
pub fn inferno(state: &mut GameState) -> (f32, f32, &'static str) {
    let rapid_lasers = repeat_periodic(LaserSpawner::new(2.0, 1.0, 45.0, 10.0), 96, 2.0, 0.25);
    let snare_bombs = repeat_periodic(BombSideSpawner::new(12, 200.0, 12.5, 2.0), 7, 4.0, 4.0);
    let drops = 12;
    let padding = 50.0;

    let rain = (0..=drops).map(|i|GSEvent(30.0,
        Box::new(move |gs: &mut UpdateAccumulator, _|gs.obst(Periodic::new(screen_height() as usize / 32 + 1, 0.125, Periodic::linear(
            2.0, 2.0, 0.25,
            vec2(i as f32 * ((screen_width() - padding * 2.0) / drops as f32) + padding, 0.0), vec2(0.0, 32.0), vec2(30.0, 30.0),
            0.0
        ))))
    )).collect::<Vec<GSEvent>>();
    let rise = (0..=drops).map(|i|GSEvent(62.0,
        Box::new(move |gs: &mut UpdateAccumulator, _|gs.obst(Periodic::new(screen_height() as usize / 32 + 3, 0.125, Periodic::linear(
            2.0, 2.0, 0.25,
            vec2(i as f32 * ((screen_width() - padding * 2.0) / drops as f32) + padding, screen_height()), vec2(0.0, -32.0), vec2(30.0, 30.0),
            0.0
        ))))
    )).collect::<Vec<GSEvent>>();

    let slam = vec![GSEvent(26.0, Box::new(|accum: &mut UpdateAccumulator, smargs| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -30.0), vec2(screen_width() / 2.0, screen_height() + 30.0), 100.0, 2.0, 4.0, 0.1, vec2(0.0, 30.0), 0.0))
    }))];

    let spiralsurge = [
        GSEvent(48.5, Box::new(|ac: &mut UpdateAccumulator, _|ac.obst(Periodic::new(400, 0.005, Periodic::rect_trail(
            2.0, 2.0, 0.125, |i| {
                let s = (i as f32).sqrt() * 1.15;
                let sr = (i as f32 - 1.0).sqrt() * 1.15;
                let spi = s + PI * (i as f32);
                let scent = vec2(screen_width(), screen_height()) / 2.0;
                (vec2(spi.sin(), spi.cos()) * s * 25.0 + scent, vec2(30.0, 30.0), sr)
            }
        )))))
    ];

    state.clear_events();
    state.add_events(
        clone_offset(&rapid_lasers, 0.0).into_iter()
        .chain(remove(clone_offset(&rapid_lasers, 32.0), 47.5, 49.75))
        .chain(clone_offset(&snare_bombs, 32.0))
        .chain(clone_offset(&snare_bombs, 0.0))
        .chain([GSEvent(4.0, Box::new(|gs: &mut UpdateAccumulator, _|{
            gs.fg(ORANGE);
            gs.bg(cmul(ORANGE, 0.2));
            gs.float(20.0);
        }))])
        .chain(rain)
        .chain(clone_offset(&slam, 32.0))
        .chain(slam)
        .chain(spiralsurge)
        .chain(rise)
    );
    (0.0, 170.0, "music/inferno.mp3")
}

/// Meganeko - Moonlight Sonata (3rd Movement) Remix
pub fn moonlight_sonata(state: &mut GameState) -> (f32, f32, &'static str) {
    state.clear_events();
    let bpm = 177.5;

    // Bombs
    state.add_event(GSEvent(-11.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(Bomb::new(Vec2::ZERO, Vec2::ZERO, 2.0, 50, 200.0, 5.0, Box::new(Bomb::pellet_spawner)));
        accum.obst(Bomb::new(screen_size(), screen_size(), 2.0, 50, 200.0, 5.0, Box::new(Bomb::pellet_spawner)));
        accum.obst(Bomb::new(screen(0.0, 1.0), screen(0.0, 1.0), 2.0, 50, 200.0, 5.0, Box::new(Bomb::pellet_spawner)));
        accum.obst(Bomb::new(screen(1.0, 0.0), screen(1.0, 0.0), 2.0, 50, 200.0, 5.0, Box::new(Bomb::pellet_spawner)));
    })));
    state.add_events(repeat_periodic(|accum: &mut UpdateAccumulator, _| {
        accum.obst(Bomb::new(vec2(screen_width(), screen_height() / 2.0), vec2(screen_width() - 200.0, screen_height() / 2.0), 2.0, 8, 250.0, 6.0, Box::new(Bomb::pellet_spawner)))
    }, 4, -2.0, 1.0));
    state.add_events(repeat_periodic(|accum: &mut UpdateAccumulator, _| {
        accum.obst(Bomb::new(vec2(0.0, screen_height() / 2.0), vec2(200.0, screen_height() / 2.0), 2.0, 8, 250.0, 6.0, Box::new(Bomb::pellet_spawner)))
    }, 4, 2.0, 1.0));
    state.add_events(repeat_periodic(|accum: &mut UpdateAccumulator, _| {
        accum.obst(Bomb::new(vec2(screen_width() / 2.0, 0.0), vec2(screen_width() / 2.0, 200.0), 2.0, 8, 250.0, 6.0, Box::new(Bomb::pellet_spawner)))
    }, 4, 6.0, 1.0));
    
    state.add_events(repeat_periodic(|accum: &mut UpdateAccumulator, _| {
        accum.obst(Bomb::new(vec2(screen_width() / 2.0, screen_height()), vec2(screen_width() / 2.0, screen_height() - 200.0), 2.0, 8, 250.0, 6.0, Box::new(Bomb::pellet_spawner)))
    }, 4, 10.0, 1.0));

    state.add_events(repeat_periodic(BombSideSpawner::new(16, 300.0, 10.0, 2.0), 24, 14.0, 1.0));

    // Lasers
    state.add_events(repeat_periodic(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(gen_range(0.0, screen_width()), -50.0), vec2(gen_range(0.0, screen_width()), screen_height() + 50.0), 25.0, 4.0, 2.0, 0.2, vec2(0.0, 20.0), 0.0));
        accum.obst(SlamLaser::new(vec2(gen_range(0.0, screen_width()), -50.0), vec2(gen_range(0.0, screen_width()), screen_height() + 50.0), 25.0, 4.0, 2.0, 0.2, vec2(0.0, 20.0), 0.0));
    }, 24, 12.0, 1.0));
    state.add_event(GSEvent(36.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -50.0), vec2(screen_width() / 2.0, screen_height() + 50.0), 100.0, 4.0, 6.0, 0.2, vec2(0.0, 20.0), 10.0));
    })));
    state.add_event(GSEvent(38.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -50.0), vec2(screen_width() / 2.0, screen_height() + 50.0), 200.0, 4.0, 6.0, 0.2, vec2(-10.0, 20.0), 40.0));
    })));
    state.add_event(GSEvent(40.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -50.0), vec2(screen_width() / 2.0, screen_height() + 50.0), 400.0, 4.0, 6.0, 0.2, vec2(10.0, 20.0), 100.0));
    })));
    
    let mut quick_slam = repeat_periodic(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(gen_range(0.0, screen_width()), -50.0), vec2(gen_range(0.0, screen_width()), screen_height() + 50.0), 50.0, 4.0, 2.0, 0.2, vec2(0.0, 20.0), 0.0));
    }, 4, 26.0, 0.5);
    state.add_events(clone_offset(&quick_slam, 8.0));
    state.add_events(clone_offset(&quick_slam, 4.0));
    state.add_events(quick_slam);
    state.add_events(repeat_periodic(LaserSpawner::new(2.0, 1.0, 45.0, 30.0), 48, 58.0, 0.25));

    state.add_event(GSEvent(68.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -250.0), vec2(screen_width() / 2.0, screen_height() + 250.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 100.0).leave_time(1.0));
    })));
    state.add_event(GSEvent(70.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0 - screen_height() / 2.0 - 250.0, -250.0), vec2(screen_width() / 2.0 + screen_height() / 2.0 + 250.0, screen_height() + 250.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 100.0).leave_time(1.0));
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0 + screen_height() / 2.0 + 250.0, -250.0), vec2(screen_width() / 2.0 - screen_height() / 2.0 - 250.0, screen_height() + 250.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 0.0).leave_time(1.0));
    })));
    state.add_event(GSEvent(72.0, Box::new(|accum: &mut UpdateAccumulator, _|accum.fg(RED))));
    state.add_event(GSEvent(72.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(-250.0, screen_height() / 4.0), vec2(screen_width() + 250.0, screen_height() / 4.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 100.0).leave_time(1.0));
        accum.obst(SlamLaser::new(vec2(-250.0, screen_height() / 4.0 * 3.0), vec2(screen_width() + 250.0, screen_height() / 4.0 * 3.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 0.0).leave_time(1.0));
    })));
    state.add_event(GSEvent(74.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(100.0, -250.0), vec2(100.0, screen_height() + 250.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 100.0).leave_time(1.0));
        accum.obst(SlamLaser::new(vec2(screen_width() - 100.0, -250.0), vec2(screen_width() - 100.0, screen_height() + 250.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 0.0).leave_time(1.0));
        accum.obst(SlamLaser::new(vec2(-250.0, screen_height() / 2.0), vec2(screen_width() + 250.0, screen_height() / 2.0), 200.0, 4.0, 1.0, 0.2, vec2(10.0, 20.0), 0.0).leave_time(1.0));
    })));
    state.add_event(GSEvent(76.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -250.0), vec2(screen_width() / 2.0, screen_height() + 250.0), 400.0, 4.0, 4.0, 0.2, vec2(10.0, 20.0), 150.0).leave_time(1.0));
    })));
    state.add_event(GSEvent(80.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(Vec2::ZERO, screen_size(), 400.0, 4.0, 4.0, 0.2, vec2(10.0, 20.0), 150.0).leave_time(1.0));
        accum.obst(SlamLaser::new(vec2(screen_width(), 0.0), vec2(0.0, screen_height()), 400.0, 4.0, 4.0, 0.2, vec2(10.0, 20.0), 0.0).leave_time(1.0));
    })));
    state.add_event(GSEvent(84.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(SlamLaser::new(vec2(screen_width() / 2.0, -400.0), vec2(screen_width() / 2.0, screen_height() + 400.0), 800.0, 4.0, 16.0, 0.2, vec2(10.0, 20.0), 400.0).leave_time(16.0));
    })));

    // Rotatable rectangles
    state.add_event(GSEvent(8.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(RotatingRect {
            center: screen_center(),
            size: vec2(screen_width() * 2.0, 50.0),
            rot: 0.0,
            warning_time: 8.0,
            show_time: 24.0,
            current_time: 0.0,
            grow_time: 1.0,
            rpb: 0.05
        });
        accum.obst(RotatingRect {
            center: screen_center(),
            size: vec2(50.0, screen_height() * 2.0),
            rot: 0.0,
            warning_time: 8.0,
            show_time: 24.0,
            current_time: 0.0,
            grow_time: 1.0,
            rpb: 0.05
        });
        accum.obst(RotatingRect {
            center: screen_center(),
            size: vec2(250.0, 250.0),
            rot: 0.0,
            warning_time: 8.0,
            show_time: 24.0,
            current_time: 0.0,
            grow_time: 1.0,
            rpb: 0.05
        });
        accum.obst(SlamLaser::new(vec2(100.0, -50.0), vec2(100.0, screen_height() + 50.0), 200.0, 8.0, 24.0, 0.2, Vec2::ZERO, 25.0));
        accum.obst(SlamLaser::new(vec2(screen_width() - 100.0, -50.0), vec2(screen_width() - 100.0, screen_height() + 50.0), 200.0, 8.0, 24.0, 0.2, Vec2::ZERO, 25.0));
    })));

    // Trails
    state.add_event(GSEvent(40.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        for i in 0..15 {
            accum.obst(Periodic::new(50, 0.25, Periodic::linear(2.0, 4.0, 0.25, vec2(screen_width() / 2.0 + 209.0, i as f32 * screen_height() / 14.0), vec2(20.0, 0.0), vec2(18.0, 18.0), 0.0)));
            accum.obst(Periodic::new(50, 0.25, Periodic::linear(2.0, 4.0, 0.25, vec2(screen_width() / 2.0 - 209.0, i as f32 * screen_height() / 14.0), vec2(-20.0, 0.0), vec2(18.0, 18.0), 0.0)));
        }
    })));

    // chiptune blips
    state.add_event(GSEvent(-23.1, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(Periodic::new(28, 0.375, Box::new(|ac: &mut UpdateAccumulator, _| {
            for i in 0..8 {
                ac.obst(RotatableRect {
                    center: floor_vec(rand_vec(Vec2::ZERO, screen_size()), vec2(20.0, 20.0)),
                    size: vec2(20.0, 20.0),
                    rot: 0.0,
                    warning_time: 4.0,
                    show_time: 2.0,
                    current_time: 0.0,
                    grow_time: 0.25
                });
            }
        })));
    })));

    // Color changes
    state.add_event(GSEvent(16.0, Box::new(|gs: &mut UpdateAccumulator, _| gs.smi(|state: &mut GameState, _| {
        state.bg_color = Box::new(|f: f32| cmul(gay(f), (-(f) % 1.0 + 1.0) * 0.375 + 0.125));
        state.fg_color = Box::new(|f: f32| mix(gay(f + FRAC_PI_2), WHITE, 0.5));
    }))));
    state.add_event(GSEvent(40.0, Box::new(|gs: &mut UpdateAccumulator, _| gs.smi(|state: &mut GameState, _| {
        state.bg_color = Box::new(|f: f32| cmul(gay(f), (-(f) % 2.0 + 2.0) * 0.375 / 1.5 + 0.125));
    }))));
    state.add_event(GSEvent(46.0, Box::new(|gs: &mut UpdateAccumulator, _| {
        gs.bg(BLACK);
        gs.obst(Periodic::new(80, 0.125, Box::new(|accum: &mut UpdateAccumulator, smargs: ModifyArgs| {
            accum.obst(Pellet::new(vec2(screen_width() / 2.0, screen_height() - smargs.step as f32 * screen_height() / 80.0), vec2((smargs.step as f32).sin(), (smargs.step as f32).cos()) * 150.0, 12.5));
        })))
    })));
    state.add_event(GSEvent(56.0, Box::new(|gs: &mut UpdateAccumulator, _| {
        gs.smi(|state: &mut GameState, _| {
            state.fg_color = Box::new(|f: f32| cmul(WHITE, (f * TAU * 2.0).sin() / 4.0 + 0.75));
        });
    })));

    (-9.7327210884 * bpm / 60.0, bpm, "./music/moonlight_sonata.mp3")
}

/// Aperture Science Psychoacoustic Laboratories - Friendly Faith Plate
pub fn friendly_faith_plate(state: &mut GameState) -> (f32, f32, &'static str) {
    let bpm = 120.0;
    state.add_event(GSEvent(NEG_INFINITY, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.fg(cmul(WHITE, 0.75));
    })));
    state.add_events(vec![].into_iter()
        .chain(rep_off([
            0.0, 0.25, 0.5, 1.0, 1.5, 1.75,
            2.0, 2.5, 2.75, 3.25, 3.5,
            4.0, 4.25, 4.5, 4.75, 5.0, 5.25, 5.5, 5.75,
            6.0, 6.25, 6.5, 7.0, 7.25, 7.5
        ], 4, 8.0).into_iter().map(|n|
            GSEvent(n - 2.0, Box::new(move |accum: &mut UpdateAccumulator, _| {
                let w = screen_width();
                for _ in 0..1 {
                    accum.obst(GrowLaser::new(
                        vec2(gen_range(w, w * 3.0), -20.0 - screen_height()),
                        vec2(gen_range(-w * 2.0, 0.0), screen_height() * 2.0 + 20.0),
                        50.0, 2.0, 1.0, Vec2::ZERO)
                            .grow_time(0.125)
                            .fade_in(0.125)
                            .fade_opacity(0.25)
                    )
                }
            }))
        ))
        .chain(repeat_periodic(|accum: &mut UpdateAccumulator, _| {
            for i in 0..2 {
                let pos = vec2(screen_width(), gen_range(screen_height() * 0.1, screen_height() * 0.9));
                accum.obst(Bomb::new(
                    pos, pos + vec2(-80.0, gen_range(-50.0, 50.0)),
                    1.0, 20, 400.0, 5.0, Box::new(Bomb::pellet_spawner)
                ))
            }
        }, 32, 31.0, 1.0))
    );
    (-2.05, bpm, "music/[120] friendly_faith_plate.mp3")
}

/// Cowbell Cult - Smoke (feat. JOEHDAH)
pub fn smoke(state: &mut GameState) -> (f32, f32, &'static str) {
    let bpm = 74.0;
    let mult = 1.1;
    state.add_events([
        GSEvent(-1.0, Box::new(|accum: &mut UpdateAccumulator, _| {
            let slsp = 200.0;
            let slrd = 15.0;

            let fsp = 300.0;
            let frd = 10.0;
            accum.obst(SmokeProj::default()
                .smevs(tev_rep(vec![
                    (0.0, SmokeEvent::SPulse(20.0)), (0.0, SmokeEvent::Pellets(20, slsp, slrd, 0.0)),
                    (1.0, SmokeEvent::Pellets(8, slsp, slrd, 0.2)),
                    (1.5, SmokeEvent::Pellets(4, slsp, slrd, 0.0)),
                    (1.75, SmokeEvent::SPulse(20.0)), (1.75, SmokeEvent::Pellets(12, slsp, slrd, 0.0)),
                    (2.5, SmokeEvent::SPulse(20.0)), (2.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)),
                    (3.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ], 6, 4.0))
                .smevs(tev_rep(vec![
                    (0.5, SmokeEvent::PelletSpinner(16, fsp, frd, 0.0, 32.0)),
                ], 4, 4.0))
                .smevs([
                    (8.25, SmokeEvent::SPulse(20.0)), (8.25, SmokeEvent::Pellets(10, slsp, slrd, 0.5)),

                    (12.0, SmokeEvent::Lasers(6, 0.0)),
                    (12.25, SmokeEvent::Lasers(6, 0.25)),

                    (24.0, SmokeEvent::Pellets(8, slsp, slrd, 0.0)),
                    (26.5, SmokeEvent::SPulse(20.0)), (26.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)),
                    (27.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ])
                .smevs([
                    (28.0, SmokeEvent::SPulse(20.0)), (28.0, SmokeEvent::Pellets(20, slsp, slrd, 0.0)),
                    (29.0, SmokeEvent::Pellets(8, slsp, slrd, 0.2)),
                    (29.5, SmokeEvent::Pellets(4, slsp, slrd, 0.0)),
                    (29.75, SmokeEvent::SPulse(20.0)), (29.75, SmokeEvent::Pellets(12, slsp, slrd, 0.0)),
                    (30.5, SmokeEvent::SPulse(20.0)), (30.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)),
                    (31.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ])
                .smevs(tev_rep(vec![
                    (15.0,  SmokeEvent::Lasers(4, 0.0 * 7.0 / 24.0)),
                    (15.25, SmokeEvent::Lasers(4, 1.0 * 7.0 / 24.0)),
                    (15.5,  SmokeEvent::Lasers(4, 2.0 * 7.0 / 24.0)),
                    (15.75, SmokeEvent::Lasers(4, 3.0 * 7.0 / 24.0)),
                    (16.25, SmokeEvent::Lasers(4, 4.0 * 7.0 / 24.0)),
                    (17.0,  SmokeEvent::Lasers(4, 5.0 * 7.0 / 24.0)),

                    (19.0,  SmokeEvent::Lasers(3, 0.0 * 2.0 / 21.0)),
                    (19.25, SmokeEvent::Lasers(3, 1.0 * 2.0 / 21.0)),
                    (19.5,  SmokeEvent::Lasers(3, 2.0 * 2.0 / 21.0)),
                    (20.0,  SmokeEvent::Lasers(3, 3.0 * 2.0 / 21.0)),
                    (20.75, SmokeEvent::Lasers(3, 4.0 * 2.0 / 21.0)),
                    (21.5,  SmokeEvent::Lasers(3, 5.0 * 2.0 / 21.0)),
                    (22.25, SmokeEvent::Lasers(3, 6.0 * 2.0 / 21.0)),
                ], 2, 8.0)).sort()
            );
        })),

        GSEvent(30.0, Box::new(|accum: &mut UpdateAccumulator, _| {
            let slsp = 200.0;
            let slrd = 15.0;

            let fsp = 300.0;
            let frd = 10.0;
            
            let dual = { SmokeProj::default()
                .warning_time(2.0)
                .disp_amp(150.0)
                .show_time(18.0)
                .leave_time(2.0)
                .smevs(tev_rep(vec![
                    (0.0, SmokeEvent::Pulse), (0.0, SmokeEvent::Pellets(20, slsp, slrd, 0.0)),
                    (1.0, SmokeEvent::Pellets(8, slsp, slrd, 0.2)),
                    (1.5, SmokeEvent::Pellets(4, slsp, slrd, 0.0)),
                    (1.75, SmokeEvent::Pulse), (1.75, SmokeEvent::Pellets(12, slsp, slrd, 0.0)),
                    (2.5, SmokeEvent::Pulse), (2.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)),
                    (3.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ], 2, 4.0))
                .smevs(tev_rep(vec![
                    (8.0, SmokeEvent::Pellets(20, slsp, slrd, 0.0)),
                    (9.5, SmokeEvent::Pellets(5, slsp, slrd, 0.0)),
                    (9.75, SmokeEvent::Pellets(12, slsp, slrd, 0.0)),
                    (10.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)),
                    (11.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ], 2, 4.0)).sort()
            };

            accum.obst(dual.clone());
            accum.obst(dual.disp_phase(0.5));
        })),

        GSEvent(48.0, Box::new(move |accum: &mut UpdateAccumulator, _| {
            let slsp = 200.0 * mult;
            let slrd = 15.0;

            let fsp = 300.0 * mult;
            let frd = 10.0;
            
            let triple = { SmokeProj::default()
                .warning_time(2.0)
                .disp_amp(150.0)
                .disp_freq(mult)
                .show_time(40.0)
                .leave_time(0.5)
                .smevs(tev_rep(vec![
                    (0.0, SmokeEvent::Pulse), (0.0, SmokeEvent::Pellets(20, slsp, slrd, 0.0)),
                    (1.0, SmokeEvent::Pellets(8, slsp, slrd, 0.2)),
                    (1.5, SmokeEvent::Pellets(4, slsp, slrd, 0.0)),
                    (1.75, SmokeEvent::Pulse), (1.75, SmokeEvent::Pellets(12, slsp, slrd, 0.0)),
                    (2.5, SmokeEvent::Pulse), (2.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)),
                    (3.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ], 10, 4.0))
                .smevs(tev_rep(vec![
                    (0.5, SmokeEvent::PelletSpinner(16, fsp, frd, 0.0, 32.0)),
                ], 10, 4.0))
                .smevs(tev_rep(vec![
                    (-1.0,  SmokeEvent::Lasers(3, 0.0 * 7.0 / 24.0)),
                    (-0.75, SmokeEvent::Lasers(3, 1.0 * 7.0 / 24.0)),
                    (-0.5,  SmokeEvent::Lasers(3, 2.0 * 7.0 / 24.0)),
                    (-0.25, SmokeEvent::Lasers(3, 3.0 * 7.0 / 24.0)),
                    (0.25,  SmokeEvent::Lasers(3, 4.0 * 7.0 / 24.0)),
                    (1.0,   SmokeEvent::Lasers(3, 5.0 * 7.0 / 24.0)),

                    (3.0,  SmokeEvent::Lasers(2, 0.0 * 2.0 / 21.0)),
                    (3.25, SmokeEvent::Lasers(2, 1.0 * 2.0 / 21.0)),
                    (3.5,  SmokeEvent::Lasers(2, 2.0 * 2.0 / 21.0)),
                    (4.0,  SmokeEvent::Lasers(2, 3.0 * 2.0 / 21.0)),
                    (4.75, SmokeEvent::Lasers(2, 4.0 * 2.0 / 21.0)),
                    (5.5,  SmokeEvent::Lasers(2, 5.0 * 2.0 / 21.0)),
                    (6.25, SmokeEvent::Lasers(2, 6.0 * 2.0 / 21.0)),
                ], 5, 8.0)).sort()
            };

            accum.obst(triple.clone());
            let mut sm2 = triple.clone().disp_phase(1.0 / 3.0);
            for (_, event) in &mut sm2.events {
                if let SmokeEvent::Lasers(_, phase) = event {
                    *phase += 1.0 / 6.0;
                }
            }
            accum.obst(sm2);
            let mut sm3 = triple.disp_phase(2.0 / 3.0);
            for (_, event) in &mut sm3.events {
                if let SmokeEvent::Lasers(_, phase) = event {
                    *phase += 2.0 / 6.0;
                }
            }
            accum.obst(sm3);
        })),

        GSEvent(88.0, Box::new(|accum: &mut UpdateAccumulator, _| {
            let slsp = 200.0;
            let slrd = 15.0;

            let fsp = 300.0;
            let frd = 10.0;
            
            let single = { SmokeProj::default()
                .warning_time(2.0)
                .disp_amp(75.0)
                .show_time(16.0)
                .leave_time(0.5)
                .smevs(tev_rep(vec![
                    (0.0, SmokeEvent::SPulse(20.0)), (0.0, SmokeEvent::Pellets(20, slsp, slrd, 0.0)), (0.0, SmokeEvent::Pellets(20, slsp / 1.25, slrd, 0.5 / 20.0)),
                    (1.0, SmokeEvent::Pellets(8, slsp, slrd, 0.2)),
                    (1.5, SmokeEvent::Pellets(4, slsp, slrd, 0.0)),
                    (1.75, SmokeEvent::SPulse(20.0)), (1.75, SmokeEvent::Pellets(12, slsp, slrd, 0.0)), (1.75, SmokeEvent::Pellets(12, slsp / 1.25, slrd, 0.5 / 12.0)),
                    (2.5, SmokeEvent::SPulse(20.0)), (2.5, SmokeEvent::Pellets(16, slsp, slrd, 0.0)), (2.5, SmokeEvent::Pellets(16, slsp / 1.25, slrd, 0.5 / 16.0)),
                    (3.25, SmokeEvent::Pellets(10, slsp, slrd, 0.0)),
                ], 4, 4.0))
                .smevs(tev_rep(vec![
                    (0.5, SmokeEvent::PelletSpinner(16, fsp, frd, 0.0, 32.0)),
                ], 4, 4.0))
                .smevs(tev_rep(vec![
                    (-1.0,  SmokeEvent::Lasers(4, 0.0 * 7.0 / 24.0)),
                    (-0.75, SmokeEvent::Lasers(4, 1.0 * 7.0 / 24.0)),
                    (-0.5,  SmokeEvent::Lasers(4, 2.0 * 7.0 / 24.0)),
                    (-0.25, SmokeEvent::Lasers(4, 3.0 * 7.0 / 24.0)),
                    (0.25,  SmokeEvent::Lasers(4, 4.0 * 7.0 / 24.0)),
                    (1.0,   SmokeEvent::Lasers(4, 5.0 * 7.0 / 24.0)),

                    (3.0,  SmokeEvent::Lasers(3, 0.0 * 2.0 / 21.0)),
                    (3.25, SmokeEvent::Lasers(3, 1.0 * 2.0 / 21.0)),
                    (3.5,  SmokeEvent::Lasers(3, 2.0 * 2.0 / 21.0)),
                    (4.0,  SmokeEvent::Lasers(3, 3.0 * 2.0 / 21.0)),
                    (4.75, SmokeEvent::Lasers(3, 4.0 * 2.0 / 21.0)),
                    (5.5,  SmokeEvent::Lasers(3, 5.0 * 2.0 / 21.0)),
                    (6.25, SmokeEvent::Lasers(3, 6.0 * 2.0 / 21.0)),
                ], 2, 8.0)).sort()
            };

            accum.obst(single);
        })),
        GSEvent(f32::NEG_INFINITY, Box::new(|accum: &mut UpdateAccumulator, _| {
            accum.float(20.0);
            accum.bg(cmul(mix(WHITE, ORANGE, 0.5), 0.2));
            accum.fg(cmul(mix(WHITE, RED, 0.5), 0.6));
        }))
    ]);
    (-1.678 * bpm / 60.0, bpm, "./music/smoke.mp3")
}

/// Shirobon - Granite
pub fn granite(state: &mut GameState) -> (f32, f32, &'static str) {
    let bpm = 128.0;
    state.add_event(GSEvent(62.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.obst(GOLGrid::default()
            .dims(64, 36)
            .first_warning_time(2.0)
            .period(0.5)
            .max(64)
            .populate(400)
        );
    })));
    state.add_event(GSEvent(64.0, Box::new(|accum: &mut UpdateAccumulator, _| {
        accum.bg(cmul(SKYBLUE, 0.1));
        accum.fg(SKYBLUE);
        accum.float(40.0);
    })));
    (0.05 / bpm * 60.0, bpm, "music/granite.mp3")
}

/// Nighthawk22 - Isolation (LIMBO Remix)
pub fn isolation(state: &mut GameState) -> (f32, f32, &'static str) {
    let bpm = 200.0;
    (0.0, bpm, "music/isolation.mp3")
}
