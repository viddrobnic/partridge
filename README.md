# Partridge

Solver for Partridge Puzzle described in [this video](https://www.youtube.com/watch?v=eqyuQZHfNPQ).

Calculating all the solutions takes cca 12h on 48 core machine, so use with caution!
If you do not want to wait this long, all the solutions and rendered images are available
[on my website](https://viddrobnic.com/partridge).

## Usage

Compile the program with

```sh
cargo build --release
```

You can calculate all solutions by running

```sh
./target/release/partridge solve
```

This will output `solutions.txt` file, which contains all the solutions.
You can then render the solutions to images by running

```sh
mkdir images
./target/release/partridge render
```

> [!NOTE]
> The command generates cca 1 700 000 files in the `images` folder.

### Solutions Format

The `solutions.txt` file is in a very simple format. Each row contains a single solutions with
tile sizes separated by spaces. Number `n` represents a tile of size `n * n`.

First tile for a solution is placed in the `(x, y) = (0, 0)` (top left). Next tile is placed in the same
row on the next free `x`. If no `x` is available, we scan the next row from left to right until we find
an available free `x`.

In other words, tiles are placed by scanning rows top to bottom and finding the first available spot.

## License

The project is licensed under the [MIT License](LICENSE).
