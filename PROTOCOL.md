# The KC protocol

For KC I'll use a client-server architecture. This is because it's a multiplayer game. Therefore I need to make a protocol for the server and client to use.

## Client

There's two types of clients that can join:
1. Players
2. Spectators

The spectators are just clients observing the game and seeing each move, but players are also able to make moves.

If spectators lose connection then the game carries on, but without them. If players lose connection, a message will be send out to every other player that a player left and the game is over.
There might be implemented a delay of some specified time, where you wait for the player to reconnect.

## Server

When the server starts, it will try to setup a game of KC. It starts by searching for clients on some default port.

When every player has joined and said that they're ready, the game will start.

The server will do the setup phase of the game and give each player a capital city.

Then it will repeat this sequence until someone wins:
1. Send out the current state of the game to every player and spectator.
2. Ask the current player to make a move.
3. When a response is made, it will validate it and if invalid go back to step two.
4. Then it will apply the move to the current board, and rotate the current player.

If someone dies, that player is moved from the player list to the spectator list.
