# RustyChess

A chess engine and API built with Rust.

## Features

- Chess board representation
- Basic move validation
- RESTful API for game interaction
- Move history tracking

## Getting Started

### Prerequisites

- Rust (1.58 or newer)
- Cargo

### Installation

1. Clone the repository

```
git clone https://github.com/yourusername/rustychess.git
cd rustychess
```

2. Build the project

```
cargo build --release
```

3. Run the server

```
cargo run --release
```

## API Usage

### Create a new game

```
POST /api/games
```

Response:

```json
{
  "id": "game-uuid",
  "game": { ... }
}
```

### Make a move

```
POST /api/games/{id}/moves
```

Body:

```json
{
  "from": "e2",
  "to": "e4"
}
```

### Get game state

```
GET /api/games/{id}
```

## UI Integration

### Using with Chess GUI Applications

You can integrate RustyChess with existing chess GUI applications that support external engines:

#### Integration with Lichess Board Editor

1. Start the RustyChess server

```
cargo run --release
```

2. Use the Lichess API to create a new study and import positions from RustyChess

```bash
# Example using curl to fetch a position from RustyChess and post to Lichess
curl -X GET http://localhost:8080/api/games/{id} | \
jq -r '.fen' | \
curl -X POST -d "fen=$(cat)" https://lichess.org/api/import
```

#### Integration with Chess.js and Chessboard.js

For web applications, you can use the RustyChess API with popular JavaScript chess libraries:

```javascript
// Example integration with Chess.js and Chessboard.js
fetch("http://localhost:8080/api/games/{id}")
  .then((response) => response.json())
  .then((data) => {
    const chess = new Chess(data.fen);
    const board = Chessboard("board", {
      position: data.fen,
      onDrop: (source, target) => {
        // Send move to RustyChess API
        fetch(`http://localhost:8080/api/games/${gameId}/moves`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            from: source,
            to: target,
          }),
        });
      },
    });
  });
```

### Desktop Integration

RustyChess implements the Universal Chess Interface (UCI) protocol, allowing it to be used with GUI applications like:

- [SCID](http://scid.sourceforge.net/)
- [Arena Chess GUI](http://www.playwitharena.de/)
- [Cute Chess](https://cutechess.com/)

To set up with a UCI-compatible GUI:

1. In the GUI, add a new engine
2. Point to the RustyChess executable path
3. Configure any engine parameters as needed

## License

This project is licensed under the MIT License - see the LICENSE file for details.
