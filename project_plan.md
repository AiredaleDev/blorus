# TODO:

* BETTER UI -- Placing pieces feels very awkward. What would feel good?
* Determine how to represent local multiplayer vs online multiplayer (how can I reuse the core gameloop?)
* Define the protocol for online multiplayer.
* Will players need to portforward? Probably yes, unless I do some browser shenanigans.
* Make a main menu.

## Network protocol

Perhaps we'll keep it simple. One player hosts, the other three connect to them.
After each turn, send a copy of the board and all other players' states to all players.
If the host suddenly disconnects in the middle of the game, we might be able to recover
by making the next player whose turn it is the new host. This is made easier by distributing
copies of the board and all 4 players between everyone (except for spectators, they only get a copy of the board to look at.)
