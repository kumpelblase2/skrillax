# Loginserver

A login server for Silkroad. This handles news and patch requests to the client, as well as the actual login and handoff
to the requested game server. It currently checks the availability of game servers through a configured list with a 
healthcheck that is exposed. Periodically it will check if the server is available and the population level.

The current implementation already handles everything I really care about and thus there's little to do here.