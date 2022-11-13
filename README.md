# `seldom_map_nav`

[![Crates.io](https://img.shields.io/crates/v/seldom_map_nav.svg)](https://crates.io/crates/seldom_map_nav)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_map_nav#license)
[![Crates.io](https://img.shields.io/crates/d/seldom_map_nav.svg)](https://crates.io/crates/seldom_map_nav)

`seldom_map_nav` is a Bevy plugin that does navmesh generation, pathfinding, and navigation
for tilemaps. Navmesh generation is available without Bevy dependency. It is agnostic
to the solution you use for your tilemaps, but doesn't require much glue. It is also agnostic
to your position type, as long as it implements `Position2<Position = Vec2>`
from [`seldom_interop`](https://github.com/Seldom-SE/seldom_interop) (ex. `Transform`).
It's compatible with [`seldom_state`](https://github.com/Seldom-SE/seldom_state)
with the `state` feature.

## Features

* Navmesh generation for finite, square tilemaps
* Awareness of navigator physical size
* Bevy plugin for pathfinding and navigation
* Integration with `seldom_state`

## Future Work

This crate is currently is maintenance mode, so I'm not currently adding new features.

- [ ] Tiles that can be pathed over in certain situations, such as doors
- [ ] Tiles that cannot be pathed over, but do not need clearance generated, such as holes

The generated paths are not always optimal, even with the greatest quality settings,
but I do not plan to fix this myself. If possible, I may switch dependencies to improve this,
though.

## [`seldom_state`](https://github.com/Seldom-SE/seldom_state) Compatibility

The `Pathfind` component and `NavBundle` work as states without enabling the `state` feature.
However, if the `state` feature is enabled, it will trigger the `DoneTrigger` when it is done
navigating (if it reaches the destination or cannot find a path).

## Usage

Add to your `Cargo.toml`

```toml
# Replace * with your desired version
[dependencies]
seldom_map_nav = "*"
```

To generate navmeshes without Bevy integration, disable the `bevy` feature
and use `Navmeshes::generate` or `seldom_map_nav::mesh::generate_navmesh`.
See the `no_bevy.rs` example.

To generate paths without using the built-in navigation, add the `MapNavPlugin` to your app,
add the `Navmeshes` component to your tilemap (or some other entity), and add
the `Pathfind` component to your navigating entity. To use the built-in navigation, also add
the `Nav` component to your navigating entity. See the `nav.rs` example. If you are having trouble
getting it to generate a path, enable the `log` feature, and it might tell you what's wrong.

If you need help, feel free to ping me
on [the Bevy Discord server](https://discord.com/invite/bevy) (`@Seldom`)! If any of the docs
need improvement, feel free to submit an issue or pr!

## Compatibility

| Bevy | `seldom_state` | `seldom_map_nav` |
| ---- | -------------- | ---------------- |
| 0.9  | 0.3            | 0.2              |
| 0.8  | 0.2            | 0.1              |

## License

`seldom_map_nav` is dual-licensed under MIT and Apache 2.0 at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.
