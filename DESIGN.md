# Design Notes

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
