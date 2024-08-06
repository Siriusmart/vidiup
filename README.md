# vidiup

Google (YouTube) is actively trying to block and disable Invidious instances, public listings such as [api.invidious.io](https://api.invidious.io/) are no longer feasible, as it makes Google's job of creating a blacklist for accessing a lot easier.

Public listings is not the way to go, Vidiup approaches the problem differently, instead of publicly displaying all healthy instances at once. Only 4 instances will be visible to all users at a time, it is the same idea on why you shouldn't show you entire hand to your opponents during Uno.

You can probably figure out the API specs by yourself, if you look into the `/api` directory. But here they are
- `/api/v1/add?region=[region]&instance[instance]`
- `/api/v1/get?region=(region?)`
- `/api/v1/regions`
- `/api/v1/stats`

I got an instance running at [vidiup.siri.sh](https://vidiup.siri.sh).

## self host

Having more than one instance kind of defeats the point of keeping the instances secret, but here's how you do it.

1. Git clone and build the project.
2. Copy files from `/template/storage` to `~/.local/share/vidiup`, `/template/config` to `~/.config/vidiup`
3. Open `sampleset.json` and add a video, playlist, channel ***ID***, and a search term used for testing.
4. Run it, it should work now. If it doesn't ask me for help.
