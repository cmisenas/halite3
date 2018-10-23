#[macro_use]
extern crate lazy_static;
extern crate rand;

use hlt::command::Command;
use hlt::game::Game;
use hlt::log::Log;
use hlt::navi::Navi;
use hlt::position::Position;
use hlt::ship::Ship;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod hlt;

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
        let mut current_positions: Vec<Position> = Vec::new();
        let mut future_positions: Vec<Position> = Vec::new();
        let mut own_ships: Vec<&Ship> = Vec::new();
        for ship_id in &me.ship_ids {
          own_ships.push(&game.ships[ship_id])
        }
        // Calculate moves first for ships with least amount of moves possible
        own_ships.sort_by(|ship_a, ship_b| navi.get_total_safe_moves(ship_a.position).cmp(&navi.get_total_safe_moves(ship_b.position)));

        for ship in own_ships {
            let cell = map.at_entity(ship);

            Log::log(&format!("For ship in: {}, {}", ship.position.x, ship.position.y));
            let command = if ship.halite > 900 {
                let shipyard_direction = navi.better_navigate(&ship, &me.shipyard.position, &me.ship_ids, &future_positions, &current_positions);
                let future_position = ship.position.directional_offset(shipyard_direction);
                current_positions.push(ship.position);
                future_positions.push(future_position);
                Log::log(&format!("Move towards shipyard: x: {}, y: {}", future_position.x, future_position.y));
                ship.move_ship(shipyard_direction)
            } else if cell.halite > 0 && navi.is_smart_safe(&ship.position, &ship.position, &me.ship_ids, &future_positions, &current_positions)  {
                Log::log(&format!("Stay still: {}", cell.halite));
                current_positions.push(ship.position);
                future_positions.push(ship.position);
                ship.stay_still()
            } else {
                let mut possible_positions = ship.position.get_surrounding_cardinals();
                possible_positions.sort_by(|position_a, position_b| map.at_position(position_b).halite.cmp(&map.at_position(position_a).halite));
                let best_position = possible_positions.iter().find(|position| navi.is_smart_safe(position, &ship.position, &me.ship_ids, &future_positions, &current_positions));
                Log::log(&format!("Number of possible_positions: {}", possible_positions.len()));
                match best_position {
                  Some(position) => {
                    Log::log(&format!("Best position: {}, {}", position.x, position.y));
                    current_positions.push(ship.position);
                    future_positions.push(*position);
                    ship.move_ship(ship.position.get_direction_to_position(position))
                  },
                  None => {
                    Log::log("Stay still no best move!");
                    current_positions.push(ship.position);
                    future_positions.push(ship.position);
                    ship.stay_still()
                  },
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
