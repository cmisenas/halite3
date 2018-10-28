Description of solution:

- Each round:
  - We look at each ship:
    - If ship has > 900 or has been sent home or there are few turns left in the game
      - Send home
        - If the distance to home is 1
          - If the occupier of shipyard is my own ship,
            - Stay still
          - Else
            - Ram that jerk
        - Else
          - Navigate towards home
      - Add to current and future positions
    - Else if ship's current cell's halite > 9
      - Stay still
    - Else
      - Navigate towards the maximum halite
        - TODO: Make this better so that we order by:
          - number of moves
          - halite amount (must be greater than 9)
  - We spawn a new ship:
    - If shipyard is empty
    - If I have enough halite
    - If less than N turn (N is currently 200)

- Order the ships by possible safe moves
  - Consider if the ship can move as well

- better_navigate
  - Input:
    - Current position
    - Destination position
    - Current positions
    - Future positions
  - Output
    - Direction
  - If the 2 possible moves towards destination are safe
    - Move towards it
  - Else
    - If moving in the y direction but blocked
      - check which x direction is safe
    - Else if moving in the x direction but blocked
      - check which y direction is safe
    - Else
      - stay still

- is_better_safe
  - Input:
    - Current position
    - Destination position
    - Other current positions
    - Other future positions
  - Output:
    - Boolean
  - If destination position already in future positions
    - return false
  - Else if destination position equals to one other current position and current position equals to one other future position
    - return false
  - Else if future position currently occupied by anyone other than my own ships *and* future position not equal to the shipyard
    - return false
  - Else
    - return true




Observed collisions
===================
- Ships ordered first will move towards a square, next ship will stay still in that square
