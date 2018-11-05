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

const MIN_CELL_HALITE: usize = 9;
const MAX_CARGO_HALITE: usize = 900;

fn can_move(map: &GameMap, ship: &Ship) -> bool {
    (map.at_entity(ship).halite as f64 * 0.1).floor() <= ship.halite as f64
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

fn get_nearest_nonempty_cell(map: &GameMap, position: &Position, navi: &Navi, owner_ships: &Vec<ShipId>, future_positions: &Vec<Position>, current_positions: &Vec<Position>) -> Vec<Position> {
    let mut distance = 1;
    let max_distance = (map.width / 2) as i32;
    let mut nonempty_cells: Vec<Position> = Vec::new();
    loop {
        let mut distant_positions = get_cells_with_distance(map, position, distance);
        nonempty_cells = distant_positions.into_iter().filter(|pos| {
            map.at_position(pos).halite > MIN_CELL_HALITE &&
            navi.is_smart_safe(pos, pos, owner_ships, future_positions, current_positions)
        }).collect();
        distance += 1;
        if nonempty_cells.len() > 0 || distance > max_distance {
            break;
        }
    }
    if nonempty_cells.len() == 0 {
        position.get_surrounding_cardinals().into_iter().map(|position| map.normalize(&position)).collect()
    } else {
        nonempty_cells
    }
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
            distant_cells.push(map.normalize(&Position{x: center.x - (max_y - y - 1), y: center.y + (y - distance)}));
            distant_cells.push(map.normalize(&Position{x: center.x + (max_y - y - 1), y: center.y + (y - distance)}));
        }
    }
    distant_cells
}

// Normalized directions
fn better_get_near_best_moves(map: &GameMap, position: &Position, navi: &Navi, owner_ships: &Vec<ShipId>, future_positions: &Vec<Position>, current_positions: &Vec<Position>) -> Vec<Position> {
    let mut nearest_nonempty_cell = get_nearest_nonempty_cell(map, position, navi, owner_ships, future_positions, current_positions);
    nearest_nonempty_cell.sort_by(|position_a, position_b| map.at_position(position_b).halite.cmp(&map.at_position(position_a).halite));
    nearest_nonempty_cell
}

// Normalized directions
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

    Log::log(&format!("Successfully created bot! My Player ID is {}. Bot rng seed is {}.", game.my_id.0, rng_seed));

    let end_x = game.map.width/2;
    let end_y = if game.players.len() == 2 { game.map.height } else { game.map.height/2 };

    // Maybe try collecting which cells have halite 900?
    for y in 0..end_y {
        for x in 0..end_x {
            let position = Position{x: x as i32, y: y as i32};
            let cell = game.map.at_position(&position);
            if cell.halite > 500 {
                top_cells_by_halite.push(position);
            }
        }
    }

    // Maybe try convolving and finding peaks?

    Log::log(&format!("Top cells by halite: {}", top_cells_by_halite.len()));
    let test_cells = get_cells_with_distance(&game.map, &Position{x: 8, y: 16}, 9);
    for cell in test_cells {
      Log::log(&format!("Cell with distance - {:?}", cell));
    }

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
        let mut sortable_ships: Vec<&Ship> = Vec::new();
        for ship_id in &me.ship_ids {
          let ship = &game.ships[ship_id];
          if can_move(map, ship) {
            sortable_ships.push(ship);
          } else {
            own_ships.push(ship);
          }
        }
        // Have high priority for ships that can't move
        // Calculate moves first for ships with least amount of moves possible
        sortable_ships.sort_by(|ship_a, ship_b| navi.get_total_safe_moves(ship_a.position).cmp(&navi.get_total_safe_moves(ship_b.position)));
        own_ships.extend(&sortable_ships);

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

            Log::log(&format!("For ship in {:?}, can move? {}, should mine? {}, going home? {}, is full? {}", ship.position, can_move_ship, can_keep_mining, is_home_bound, ship_is_full));

            let (command, future_position) = if !can_move_ship {
                Log::log(&format!("CANNOT MOVE ship in {:?} - cargo: {}, cell: {}", ship.position, ship.halite, cell.halite));
                (ship.stay_still(), ship.position)
            } else if ship_is_full || is_home_bound || should_go_home {
                Log::log(&format!("GO HOME ship in {:?} - cargo: {}, cell: {}", ship.position, ship.halite, cell.halite));
                let shipyard_direction = if home_distance == 1 && (should_go_home || is_shipyard_empty_next_turn) {
                  // Ram into the jerk camping at my base!
                  is_shipyard_empty_next_turn = false;
                  ship.get_home_direction(nearest_base)
                } else {
                  home_bound_ships.insert(ship.id);
                  navi.better_navigate(&ship, &nearest_base, &me.ship_ids, &future_positions, &current_positions)
                };
                let future_position = map.normalize(&ship.position.directional_offset(shipyard_direction));
                Log::log(&format!("Move towards shipyard: {:?}", future_position));
                (ship.move_ship(shipyard_direction), future_position)
            } else if can_keep_mining {
                Log::log(&format!("STAY STILL ship in: {:?} - cargo: {}, cell: {}", ship.position, ship.halite, cell.halite));
                (ship.stay_still(), ship.position)
            } else {
                Log::log("All immediate possible positions are empty!");
                let mut nearest_best_possible_positions = better_get_near_best_moves(map, &ship.position, &navi, &me.ship_ids, &future_positions, &current_positions);
                Log::log(&format!("Number of best possible positions: {}", nearest_best_possible_positions.len()));
                // Set destination to the first best one
                let destination = nearest_best_possible_positions.first();
                match destination {
                  Some(dest_pos) => {
                    let best_direction = navi.better_navigate(&ship, &dest_pos, &me.ship_ids, &future_positions, &current_positions);
                    let new_pos = map.normalize(&ship.position.directional_offset(best_direction));
                    Log::log(&format!("Destination position: {:?} | Best position: {:?}, {:?}", dest_pos, new_pos, best_direction));
                    (ship.move_ship(best_direction), new_pos)
                  },
                  // This should never happen unless the entire map has been emptied
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
