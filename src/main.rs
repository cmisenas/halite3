#[macro_use]
extern crate lazy_static;
extern crate rand;

use hlt::command::Command;
use hlt::game::Game;
use hlt::log::Log;
use hlt::navi::Navi;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod hlt;

/**
fn naive_navigate(&mut self, ship: &Ship, destination: &Position) -> Direction {
}
*/

fn main() {
    let args: Vec<String> = env::args().collect();
    let rng_seed: u64 = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    };

    let mut game = Game::new();
    let mut navi = Navi::new(game.map.width, game.map.height);
    // At this point "game" variable is populated with initial map data.
    // This is a good place to do computationally expensive start-up pre-processing.
    // As soon as you call "ready" function below, the 2 second per turn timer will start.
    Game::ready("Overlord");

    Log::log(&format!("Successfully created bot! My Player ID is {}. Bot rng seed is {}.", game.my_id.0, rng_seed));

    loop {
        game.update_frame();
        navi.update_frame(&game);

        let me = &game.players[game.my_id.0];
        let map = &mut game.map;

        let mut command_queue: Vec<Command> = Vec::new();

        for ship_id in &me.ship_ids {
            let ship = &game.ships[ship_id];
            let cell = map.at_entity(ship);

            let command = if ship.halite > 900 {
                let shipyard_direction = navi.naive_navigate(ship, &me.shipyard.position);
                // maybe mark unsafe cell here
                ship.move_ship(shipyard_direction)
            } else if cell.halite > 0  {
                ship.stay_still()
            } else {
                let mut possible_positions = ship.position.get_surrounding_cardinals();
                possible_positions.sort_by(|position_a, position_b| map.at_position(position_b).halite.cmp(&map.at_position(position_a).halite));
                let best_position = possible_positions.iter().find(|position| navi.is_safe(position));
                Log::log(&format!("Number of possible_positions: {}", possible_positions.len()));
                match best_position {
                  Some(position) => {
                    Log::log(&format!("Best position: {}, {}", position.x, position.y));
                    ship.move_ship(navi.naive_navigate(ship, position))
                  },
                  None => ship.stay_still(),
                }
            };
            command_queue.push(command);
        }

        if
            game.turn_number <= 200 &&
            me.halite >= game.constants.ship_cost &&
            navi.is_safe(&me.shipyard.position)
        {
            command_queue.push(me.shipyard.spawn());
        }


        Game::end_turn(&command_queue);
    }
}
