use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use rand::{thread_rng, Rng};
use std::{
    io::{stdout, Result, Stdout, Write},
    thread, time,
    time::Duration,
};

struct World {
    player_c: u16,
    player_l: u16,
    map: Vec<(u16, u16)>,
    maxc: u16,
    deid: bool,
    next_start: u16,
    next_end: u16,
}

fn draw(mut sc: &Stdout, world: &World) -> Result<()> {
    sc.queue(Clear(ClearType::All))?;

    // draw the map
    for line in 0..world.map.len() {
        sc.queue(MoveTo(0, line as u16))?;
        sc.queue(Print("*".repeat(world.map[line].0 as usize)))?;
        sc.queue(MoveTo(world.map[line].1, line as u16))?;
        sc.queue(Print("*".repeat((world.maxc - world.map[line].1) as usize)))?;
    }

    // draw the player
    sc.queue(MoveTo(world.player_c, world.player_l))?;
    sc.queue(Print("P"))?;

    sc.flush()?;

    Ok(())
}

fn physics(mut world: World) -> Result<World> {
    // check if player deid
    if world.player_c <= world.map[world.player_l as usize].0
        || world.player_c >= world.map[world.player_l as usize].1
    {
        world.deid = true;
    }

    // shift the map to draw river
    for line in (0..world.map.len() - 1).rev() {
        world.map[line + 1] = world.map[line];
    }
    if world.next_end < world.map[0].1 {
        world.map[0].1 -= 1;
    }
    if world.next_end > world.map[0].1 {
        world.map[0].1 += 1
    }
    if world.next_start < world.map[0].0 {
        world.map[0].0 -= 1;
    }
    if world.next_start > world.map[0].0 {
        world.map[0].0 += 1
    }

    let mut rng = thread_rng();
    if rng.gen_range(0..10) > 7 {
        if world.next_start == world.map[0].0 && world.next_end == world.map[0].1 {
            world.next_start = rng.gen_range(world.map[0].0 - 5..world.map[0].1 + 5);
            world.next_end = rng.gen_range(world.map[0].0 + 5..world.map[0].1 + 5);
            if world.next_end - world.next_start <= 3 {
                world.next_start -= 3;
            }
        }
    }

    Ok(world)
}

fn main() -> Result<()> {
    // init the screen
    let mut sc = stdout();
    sc.execute(Hide)?;
    let (maxc, maxl) = size().unwrap();
    enable_raw_mode()?;

    // init the game
    let mut world = World {
        player_c: maxc / 2,
        player_l: maxl - 1,
        map: vec![((maxc / 2) - 5, (maxc / 2) + 5); maxl as usize],
        maxc: maxc,
        deid: false,
        next_end: maxc / 2 + 10,
        next_start: maxc / 2 - 10,
    };

    while !world.deid {
        // ready and apply keyboard
        if poll(Duration::from_millis(10))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            let key = read().unwrap();
            while poll(Duration::from_millis(0)).unwrap() {
                let _ = read();
            }
            match key {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('w') => {
                        if world.player_l > 1 {
                            world.player_l -= 1;
                        }
                    }
                    KeyCode::Char('s') => {
                        if world.player_l < maxl - 1 {
                            world.player_l += 1;
                        }
                    }
                    KeyCode::Char('a') => {
                        if world.player_c > 1 {
                            world.player_c -= 1;
                        }
                    }
                    KeyCode::Char('d') => {
                        if world.player_c < maxc - 1 {
                            world.player_c += 1;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            // Timeout expired and no `Event` is available
        }

        world = physics(world).unwrap();

        draw(&sc, &world)?;

        thread::sleep(time::Duration::from_millis(100));
    }
    sc.execute(Show)?;
    disable_raw_mode()?;
    sc.execute(Clear(ClearType::All))?;
    sc.execute(Print("Thanks for playing..."))?;
    Ok(())
}
