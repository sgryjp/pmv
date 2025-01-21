# Design Notes

## Terminology

- "Move action" (or "move" in short)
  - An operation to move a file from a source to a destination.

## Flags and behavior

The program's behavior according to the flags are shown below:

| `-n` | `-i` | `-v` | Verbose? | Prompt? | Moves? |
| ---- | ---- | ---- | -------- | ------- | ------ |
| -    | -    | -    | No       | No      | Yes    |
| -    | -    | x    | Yes      | No      | Yes    |
| -    | x    | -    | Yes      | Yes     | If "y" |
| x    | -    | -    | Yes      | No      | No     |
| -    | x    | x    | Yes      | Yes     | If "y" |
| x    | x    | -    | Yes      | No      | No     |
| x    | -    | x    | Yes      | No      | No     |
| x    | x    | x    | Yes      | No      | No     |

## Algorithm to sort move actions

### Unsafe cases

From the given command parameters, `pmv` collects a set of move actions. A set
of actions can contain unsafe combination of actions and/or actions in unsafe
order.

There two cases for the unsafe combination:

- (a-1) multiple actions share an identical source (e.g.: A→B, A→C)
- (a-2) multiple actions share an identical destinations (e.g.: A→C, B→C)

...and there is two cases for unsafe order:

- (b-1) the source of a move action is the destination of another action (e.g.:
  A→B, B→C)
- (b-2) the "chain" of move actions forms a circular network (e.g.: A→B, A→B)

In case of (a-1), the actions are simply impossible because we cannot _move_ a
file to multiple destination (and `pmv` is not a _copy command_). In case of
(a-2), the source of the action which will be executed after the other will be
overwritten. `pmv` must detect there cases before executing move actions. In
case of (b-1), depending on the execution order a source a move action will be
lost (in the example above list, B will be lost if the execution order is A→B
then B→C). Lastly in case of (b-2), the same problem as (b-1) occurs. Note that
unlike (b-1), we cannot resolve a "safe order" for (b-2).

### The algorithm to execute them safely

Taking all the above in mind, `pmv` processes the given multiple move actions
with the algorithm below:

1. Find any combination of move actions sharing their sources or destinations.
   If found, stop processing. Otherwise proceed to the next step. (a-1)(a-2)
2. Prepare an empty list of actions (`result`) to store safely sorted actions.
3. Select an action in the collected set of move actions (`input`).
4. Find an action of which source is the same as the selected one's destination.
5. If found, select the action and repeat from step 4 until no action satisfying
   the condition found.
6. If no action was found, make an array from the selected series of move
   actions. If the source of the first action is the same as the destination of
   the last action, make a temporary file and modify the list as below:

   - Change the destination of the last action to the temporary file.
   - Prepend an action moving the temporary file to the source of the first
     action.

   Then, append the list in reversed order to `result`. After that, remove the
   appended actions from `input`. (b-1)(b-2)

7. Repeat from step 3 until `input` becomes empty.
