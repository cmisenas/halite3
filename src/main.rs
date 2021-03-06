#[macro_use]
extern crate lazy_static;
extern crate rand;

use hlt::command::Command;
use hlt::direction::Direction;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::log::Log;
use hlt::navi::Navi;
use hlt::player::Player;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use std::collections::HashSet;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod hlt;

fn can_move(map: &GameMap, ship: &Ship) -> bool {
  (map.at_entity(ship).halite as float * 0.1) <= ship.halite as float
}

fn get_nearest_base(game: &Game, me: &Player, ship: &Ship) -> Position {
  // Get the shipyard position and all the dropoff positions in a vec
  let mut bases: Vec<Position> = Vec::new();
  let player_dropoffs = &me.dropoff_ids;
  bases.push(me.shipyard.position);
  for dropff_id in player_dropoffs {
    bases.push(game.dropoffs[&dropff_id].position);
  }
  bases.sort_by(|base_a, base_b| game.map.calculate_distance(&ship.position, &base_a).cmp(&game.map.calculate_distance(&ship.position, &base_b)));
  *bases.first().unwrap_or(&me.shipyard.position)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rng_seed: u64 = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    };

    let mut game = Game::new();
    let mut navi = Navi::new(game.map.width, game.map.height);
    let mut home_bound_ships: HashSet<ShipId> = HashSet::new();
    // At this point "game" variable is populated with initial map data.
    // This is a good place to do computationally expensive start-up pre-processing.
    // As soon as you call "ready" function below, the 2 second per turn timer will start.
    Game::ready("Overlord");
    const MIN_CELL_HALITE: usize = 0;
    const MAX_CARGO_HALITE: usize = 900;

    Log::log(&format!("Successfully created bot! My Player ID is {}. Bot rng seed is {}.", game.my_id.0, rng_seed));

    loop {
        game.update_frame();
        navi.update_frame(&game);

        let me = &game.players[game.my_id.0];
        let map = &mut game.map;
        let remaining_turns = (game.constants.max_turns - game.turn_number) as i32;
        let mut is_shipyard_empty_next_turn = true;

        let mut command_queue: Vec<Command> = Vec::new();
        let mut current_positions: Vec<Position> = Vec::new();
        let mut future_positions: Vec<Position> = Vec::new();
        let mut own_ships: Vec<&Ship> = Vec::new();
        for ship_id in &me.ship_ids {
          own_ships.push(&game.ships[ship_id]);
        }
        // Calculate moves first for ships with least amount of moves possible
        own_ships.sort_by(|ship_a, ship_b| navi.get_total_safe_moves(ship_a.position).cmp(&navi.get_total_safe_moves(ship_b.position)));

        for ship in own_ships {
            let cell = map.at_entity(ship);
            let home_distance = map.calculate_distance(&ship.position, &me.shipyard.position) as i32;
            let should_go_home = (remaining_turns - home_distance).abs() <= 5;
            current_positions.push(ship.position);
            Log::log(&format!("For ship in x: {}, y: {} | is home bound? {}", ship.position.x, ship.position.y, home_bound_ships.contains(&ship.id)));
            if ship.position.equal(&me.shipyard.position) {
              home_bound_ships.remove(&ship.id);
            }

            let (command, future_position) = if !can_move(map, ship) {
                Log::log(&format!("CANNOT MOVE ship in x: {}, y: {} - cargo: {}, cell: {}", ship.position.x, ship.position.y, ship.halite, cell.halite));
                (ship.stay_still(), ship.position)
            } else if ship.halite > MAX_CARGO_HALITE || home_bound_ships.contains(&ship.id) || should_go_home {
                let shipyard_direction = if home_distance == 1 {
                  // Ram into the jerk camping at my base!
                  is_shipyard_empty_next_turn = false;
                  if ship.position.x < me.shipyard.position.x {
                    Direction::East
                  } else if ship.position.x > me.shipyard.position.x {
                    Direction::West
                  } else if ship.position.y < me.shipyard.position.y {
                    Direction::South
                  } else {
                    Direction::North
                  }
                } else {
                  home_bound_ships.insert(ship.id);
                  navi.better_navigate(&ship, &me.shipyard.position, &me.ship_ids, &future_positions, &current_positions)
                };
                let future_position = ship.position.directional_offset(shipyard_direction);
                Log::log(&format!("Move towards shipyard: x: {}, y: {}", future_position.x, future_position.y));
                (ship.move_ship(shipyard_direction), future_position)
            } else if cell.halite > MIN_CELL_HALITE && navi.is_smart_safe(&ship.position, &ship.position, &me.ship_ids, &future_positions, &current_positions)  {
                Log::log(&format!("Stay still: {}", cell.halite));
                (ship.stay_still(), ship.position)
            } else {
                let mut possible_positions = ship.position.get_surrounding_cardinals();
                possible_positions.sort_by(|position_a, position_b| map.at_position(position_b).halite.cmp(&map.at_position(position_a).halite));
                let best_position = possible_positions.iter().find(|position| navi.is_smart_safe(position, &ship.position, &me.ship_ids, &future_positions, &current_positions));
                Log::log(&format!("Number of possible_positions: {}", possible_positions.len()));
                match best_position {
                  Some(position) => {
                    Log::log(&format!("Best position: {}, {}", position.x, position.y));
                    (ship.move_ship(ship.position.get_direction_from_position(position)), *position)
                  },
                  None => {
                    Log::log("Stay still no best move!");
                    (ship.stay_still(), ship.position)
                  },
                }
            };
            future_positions.push(future_position);
            command_queue.push(command);
        }
        Log::log(&format!("Is shipyard empty next turn? {}", is_shipyard_empty_next_turn));

        if
            game.turn_number <= 250 &&
            me.halite >= game.constants.ship_cost &&
            is_shipyard_empty_next_turn &&
            navi.is_safe(&me.shipyard.position)
        {
            command_queue.push(me.shipyard.spawn());
        }


        Game::end_turn(&command_queue);
    }
}
