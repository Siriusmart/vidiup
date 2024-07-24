I'm packing up for a trip rn, there is no time to explain.

Google (YouTube) is actively trying to block and disable Invidious instances, public listings such as [api.invidious.io](https://api.invidious.io/) are no longer feasible, as it makes Google's job of creating a blacklist for accessing a lot easier.

Public listings is not the way to go, Vidiup approaches the problem differently, instead of publicly displaying all healthy instances at once. Only 4 instances will be visible to all users at a time, it is the same idea on why you shouldn't show you entire hand to your opponents during Uno.

You can probably figure out the API specs by yourself, if you look into the `/api` directory. But here they are
- `/api/v1/add?region=[region]&instance[instance]`
- `/api/v1/get?region=(region?)`
- `/api/v1/regions`
- `/api/v1/stats`

I got an instance running at [vidiup.siri.sh](https://vidiup.siri.sh).

If you want to self host, figure it out yourself before I'm back from my trip. Just know that having more than one instance of this kinda defeats the whole point of a central index.
