#[macro_use]
extern crate lazy_static;
extern crate rand;

use hlt::command::Command;
use hlt::dropoff::Dropoff;
use hlt::DropoffId;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::log::Log;
use hlt::navi::Navi;
use hlt::player::Player;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;


mod hlt;

fn can_move(map: &GameMap, ship: &Ship) -> bool {
    (map.at_entity(ship).halite as f64 * 0.1) <= ship.halite as f64
}

fn get_nearest_base(map: &GameMap, dropoffs: &HashMap<DropoffId, Dropoff>, me: &Player, ship: &Ship) -> Position {
    // Get the shipyard position and all the dropoff positions in a vec
    let mut bases: Vec<Position> = Vec::new();
    let player_dropoffs = &me.dropoff_ids;
    bases.push(me.shipyard.position);
    for dropff_id in player_dropoffs {
        bases.push(dropoffs[&dropff_id].position);
    }
    bases.sort_by(|base_a, base_b| map.calculate_distance(&ship.position, &base_a).cmp(&map.calculate_distance(&ship.position, &base_b)));
    *bases.first().unwrap_or(&me.shipyard.position)
}

fn get_nearest_nonempty_cell(map: &GameMap, position: &Position) -> Vec<Position> {
    // We can start with 2 since 4 cardinal directions can be easily checked and get_near_best_moves can be used for that
    let mut distance = 2;
    let mut nonempty_cells: Vec<Position> = Vec::new();
    loop {
        let mut distant_positions = get_cells_with_distance(map, position, distance);
        nonempty_cells = distant_positions.into_iter().filter(|pos| map.at_position(pos).halite > 0).collect();
        distance += 1;
        if nonempty_cells.len() > 0 {
            break;
        }
    }
    nonempty_cells
}

fn get_cells_with_distance(map: &GameMap, center: &Position, distance: i32) -> Vec<Position> {
    let mut distant_cells: Vec<Position> = Vec::new();
    let max_y = distance * 2 + 1;
    for y in 0..max_y {
        if y == 0 {
            distant_cells.push(map.normalize(&Position{x: center.x, y: center.y - distance}));
        } else if y == max_y - 1 {
            distant_cells.push(map.normalize(&Position{x: center.x, y: center.y + distance}));
        } else if y <= distance {
            distant_cells.push(map.normalize(&Position{x: center.x - y, y: center.y - (distance - y)}));
            distant_cells.push(map.normalize(&Position{x: center.x + y, y: center.y - (distance - y)}));
        } else if y > distance {
            distant_cells.push(map.normalize(&Position{x: center.x - (distance - y), y: center.y + (distance - y)}));
            distant_cells.push(map.normalize(&Position{x: center.x + (distance - y), y: center.y + (distance - y)}));
        }
    }
    distant_cells
}

fn better_get_near_best_moves(map: &GameMap, position: &Position) -> Vec<Position> {
    let mut nearest_nonempty_cell = get_nearest_nonempty_cell(map, position);
    nearest_nonempty_cell.sort_by(|position_a, position_b| map.at_position(position_b).halite.cmp(&map.at_position(position_a).halite));
    nearest_nonempty_cell
}

fn get_near_best_moves(map: &GameMap, position: &Position) -> Vec<Position> {
    // Starting at distance 1 from current location, get cells
    let mut positions: Vec<Position> = position.get_surrounding_cardinals().into_iter().map(|position| map.normalize(&position)).collect();
    positions.sort_by(|position_a, position_b| map.at_position(position_b).halite.cmp(&map.at_position(position_a).halite));
    positions
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
    let mut previous_positions: HashMap<ShipId, Position> = HashMap::new();
    let mut top_cells_by_halite = Vec::new();
    // let mut peaks_by_halite = Vec::new();
    // At this point "game" variable is populated with initial map data.
    // This is a good place to do computationally expensive start-up pre-processing.
    // As soon as you call "ready" function below, the 2 second per turn timer will start.
    Game::ready("Overlord");
    const MIN_CELL_HALITE: usize = 0;
    const MAX_CARGO_HALITE: usize = 900;

    Log::log(&format!("Successfully created bot! My Player ID is {}. Bot rng seed is {}.", game.my_id.0, rng_seed));

    // Maybe try collecting which cells have halite 900?
    for y in 0..game.map.height {
        for x in 0..game.map.width {
            let position = Position{x: x as i32, y: y as i32};
            let cell = game.map.at_position(&position);
            if cell.halite > 800 {
                top_cells_by_halite.push(position);
            }
        }
    }

    // Maybe try convolving and finding peaks?

    Log::log(&format!("Top cells by halite: {}", top_cells_by_halite.len()));

    loop {
        game.update_frame();
        navi.update_frame(&game);

        let me = &game.players[game.my_id.0];
        let map = &mut game.map;
        let dropoffs = &mut game.dropoffs;
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
            let nearest_base = get_nearest_base(map, dropoffs, me, ship);
            let home_distance = map.calculate_distance(&ship.position, &nearest_base) as i32;
            let ship_at_base = ship.position.equal(&nearest_base);
            let ship_is_full = ship.halite > MAX_CARGO_HALITE;
            let should_go_home = (remaining_turns - home_distance).abs() <= 5;
            let can_move_ship = can_move(map, ship);
            let can_keep_mining = cell.halite > MIN_CELL_HALITE && navi.is_smart_safe(&ship.position, &ship.position, &me.ship_ids, &future_positions, &current_positions);

            if ship_at_base {
              home_bound_ships.remove(&ship.id);
            }
            current_positions.push(ship.position);
            previous_positions.insert(ship.id, ship.position);
            let is_home_bound = home_bound_ships.contains(&ship.id);

            Log::log(&format!("For ship in x: {}, y: {}, can move? {}, should mine? {}, going home? {}, is full? {}", ship.position.x, ship.position.y, can_move_ship, can_keep_mining, is_home_bound, ship_is_full));

            let (command, future_position) = if !can_move_ship {
                Log::log(&format!("CANNOT MOVE ship in x: {}, y: {} - cargo: {}, cell: {}", ship.position.x, ship.position.y, ship.halite, cell.halite));
                (ship.stay_still(), ship.position)
            } else if ship_is_full || is_home_bound || should_go_home {
                Log::log(&format!("GO HOME ship in x: {}, y: {} - cargo: {}, cell: {}", ship.position.x, ship.position.y, ship.halite, cell.halite));
                let shipyard_direction = if home_distance == 1 {
                  // Ram into the jerk camping at my base!
                  is_shipyard_empty_next_turn = false;
                  ship.get_home_direction(nearest_base)
                } else {
                  home_bound_ships.insert(ship.id);
                  navi.better_navigate(&ship, &nearest_base, &me.ship_ids, &future_positions, &current_positions)
                };
                let future_position = ship.position.directional_offset(shipyard_direction);
                Log::log(&format!("Move towards shipyard: x: {}, y: {}", future_position.x, future_position.y));
                (ship.move_ship(shipyard_direction), future_position)
            } else if can_keep_mining {
                Log::log(&format!("STAY STILL ship in x: {}, y: {} - cargo: {}, cell: {}", ship.position.x, ship.position.y, ship.halite, cell.halite));
                (ship.stay_still(), ship.position)
            } else {
                let mut possible_positions: Vec<Position> = get_near_best_moves(map, &ship.position);
                // if all possible positions are empty
                if possible_positions.iter().all(|pos| map.at_position(pos).halite == 0) {
                    Log::log("All immediate possible positions are empty!");
                    let mut nearest_best_possible_positions = better_get_near_best_moves(map, &ship.position);
                    Log::log(&format!("Number of best possible positions: {}", nearest_best_possible_positions.len()));
                    // Set destination to the first best one
                    let destination = nearest_best_possible_positions.first();
                    match destination {
                      Some(dest_pos) => {
                        let best_direction = navi.better_navigate(&ship, &dest_pos, &me.ship_ids, &future_positions, &current_positions);
                        (ship.move_ship(best_direction), *dest_pos)
                      },
                      // This should never happen unless the entire map has been emptied
                      None => {
                        Log::log("Stay still no best move!");
                        (ship.stay_still(), ship.position)
                      },
                    }
                } else {
                    let best_position = possible_positions.iter().find(|position|
                        navi.is_smart_safe(position, &ship.position, &me.ship_ids, &future_positions, &current_positions) &&
                        // Don't send a ship to the same cell it was before
                        // Use a non-existent position if previous position was not set for ship
                        !previous_positions.get(&ship.id).unwrap_or(&Position {x: -1, y: -1}).equal(position)
                    );
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
