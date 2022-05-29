# Agent Server

This is the actual game server where players play on. As of now, it is still very basic in what it can do. Currently, 
the following is implemented:
- Selecting a character
- Spawning in the world
- Moving (without collision)

Additionally, it currently only handles one player, i.e. players cannot interact with one another or even see each 
other. To better understand ECS for multiplayer, I still plan on implementing the following:

- actual multiplayer
- monster spawning
- simple battle

While the current code works, there are quite a few areas that I'm unhappy with the design of. One of these things is 
how character data is stored in character select and the transition over to in game with this character data. It still
seems quite messy in my eyes. Additionally, the way packets are handled when in game won't scale and I'd like to have a
better solution.