# Layering/Builder Chaining

---

***About this tutorial***

*This tutorial is free and open source, and all code uses the MIT license - so you are free to do with it as you like. My hope is that you will enjoy the tutorial, and make great games!*

*If you enjoy this and would like me to keep writing, please consider supporting [my Patreon](https://www.patreon.com/blackfuture).*

[![Hands-On Rust](./beta-webBanner.jpg)](https://pragprog.com/titles/hwrust/hands-on-rust/)

---

The last few chapters have introduced an important concept in procedural generation: chained builders. We're happily building a map, calling Wave Function Collapse to mutate the map, calling our `PrefabBuilder` to change it again, and so on. This chapter will formalize this process a bit, expand upon it, and leave you with a framework that lets you *clearly* build new maps by chaining concepts together.

## A builder-based interface

Builder chaining is a pretty profound approach to procedurally generating maps, and gives us an opportunity to clean up a lot of the code we've built thus far. We want an interface similar to the way we build entities with `Specs`: a builder, onto which we can keep chaining builders and return it as an "executor" - ready to build the maps. We also want to stop builders from doing more than one thing - they should do one thing, and do it well (that's a good principle of design; it makes debugging easier, and reduces duplication).

There are two major types of builders: those that *just* make a map (and only make sense to run once), and those that *modify* an existing map. We'll name those `InitialMapBuilder` and `MetaMapBuilder` respectively.

This gives us an idea of the syntax we want to employ:

* Our *Builder* should have:
  * ONE Initial Builder.
  * *n* Meta Builders, that run in order.

It makes sense then that the builder should have a `start_with` method that accepts the first map, and additional `with` methods to chain builders. The builders should be stored in a container that preserves the order in which they were added - a vector being the obvious choice.

It would also make sense to no longer make individual builders responsible for setting up their predecessors; ideally, a builder shouldn't *have* to know anything about the process beyond what *it* does. So we need to abstract the process, and support snapshotting (so you can view your procedural generation progress) along the way.

## Shared map state - the `BuilderMap`

Rather than each builder defining their own copies of shared data, it would make sense to put the shared data in one place - and pass it around the chain as needed. So we'll start by defining some new structures and interfaces. First of all, we'll make `BuilderMap` in `map_builders/mod.rs`:

```rust
pub struct BuilderMap {
    pub spawn_list : Vec<(usize, String)>,
    pub map : Map,
    pub starting_position : Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history : Vec<Map>
}
```

You'll notice that this has all of the data we've been building into each map builder - and nothing else. It's intentionally generic - we'll be passing it to builders, and letting them work on it. Notice that all the fields are *public* - that's because we're passing it around, and there's a good chance that anything that touches it will need to access any/all of its contents.

The `BuilderMap` also needs to facilitate the task of taking snapshots for debugger viewing of maps as we work on algorithms. We're going to put one function into `BuilderMap` - to handle snapshotting development:

```rust
impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}
```

This is the *same* as the `take_snapshot` code we've been mixing into our builders. Since we're using a central repository of map building knowledge, we can promote it to apply to *all* our builders.

## The `BuilderChain` - master builder to manage map creation

Previously, we've passed `MapBuilder` classes around, each capable of building previous maps. Since we've concluded that this is a poor idea, and defined the syntax we *want*, we'll make a replacement. The `BuilderChain` is a *master* builder - it controls the whole build process. To this end, we'll add the `BuilderChain` type:

```rust
pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data : BuilderMap
}
```

This is a more complicated structure, so let's go through it:

* `starter` is an `Option`, so we know if there is one. Not having a first step (a map that doesn't refer to other maps) would be an error condition, so we'll track it. We're referencing a new trait, `InitialMapBuilder`; we'll get to that in a moment.
* `builders` is a vector of `MetaMapBuilders`, another new trait (and again - we'll get to it in a moment). These are builders that operate on the results of previous maps.
* `build_data` is a public variable (anyone can read/write it), containing the `BuilderMap` we just made.

We'll implement some functions to support it. First up, a *constructor*:

```rust
impl BuilderChain {
    pub fn new(new_depth : i32) -> BuilderChain {
        BuilderChain{
            starter: None,
            builders: Vec::new(),
            build_data : BuilderMap {
                spawn_list: Vec::new(),
                map: Map::new(new_depth),
                starting_position: None,
                rooms: None,
                history : Vec::new()
            }
        }
    }
    ...
```

This is pretty simple: it makes a new `BuilderChain` with default values for everything. Now, lets permit our users to add a *starting map* to the chain. (A starting map is a first step that doesn't require a previous map as input, and results in a usable map structure which we may then modify):

```rust
...
pub fn start_with(&mut self, starter : Box<dyn InitialMapBuilder>) {
    match self.starter {
        None => self.starter = Some(starter),
        Some(_) => panic!("You can only have one starting builder.")
    };
}
...
```

There's one new concept in here: `panic!`. If the user tries to add a second starting builder, we'll crash - because that doesn't make any sense. You'd simply be overwriting your previous steps, which is a giant waste of time! We'll also permit the user to add meta-builders:

```rust
...
pub fn with(&mut self, metabuilder : Box<dyn MetaMapBuilder>) {
    self.builders.push(metabuilder);
}
...
```

This is very simple: we simply add the meta-builder to the builder vector. Since vectors remain in the order in which you add to them, your operations will remain sorted appropriately. Finally, we'll implement a function to actually construct the map:

```rust
pub fn build_map(&mut self, rng : &mut rltk::RandomNumberGenerator) {
    match &mut self.starter {
        None => panic!("Cannot run a map builder chain without a starting build system"),
        Some(starter) => {
            // Build the starting map
            starter.build_map(rng, &mut self.build_data);
        }
    }

    // Build additional layers in turn
    for metabuilder in self.builders.iter_mut() {
        metabuilder.build_map(rng, &mut self.build_data);
    }
}
```

Let's walk through the steps here:

1. We `match` on our starting map. If there isn't one, we panic - and crash the program with a message that you *have* to set a starting builder.
2. We call `build_map` on the starting map.
3. For each meta-builder, we call `build_map` on it - in the order specified.

That's not a bad syntax! It should enable us to chain builders together, and provide the required overview for constructing complicated, layered maps.

## New Traits - `InitialMapBuilder` and `MetaMapBuilder`

Lets look at the two trait interfaces we've defined, `InitialMapBuilder` and `MetaMapBuilder`. We made them separate types to force the user to only pick *one* starting builder, and not try to put any starting builders in the list of modification layers. The implementation for them is the same:

```rust
pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}

pub trait MetaMapBuilder {    
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap);
}
```

`build_map` takes a random-number generator (so we stop creating new ones everywhere!), and a mutable reference to the `BuilderMap` we are working on. So instead of each builder optionally calling the previous one, we're passing along state as we work on it.

## Spawn Function

We'll also want to implement our spawning system:

```rust
pub fn spawn_entities(&mut self, ecs : &mut World) {
    for entity in self.build_data.spawn_list.iter() {
        spawner::spawn_entity(ecs, &(&entity.0, &entity.1));
    }
}
```

This is almost exactly the same code as our previous spawner in `MapBuilder`, but instead we're spawning from the `spawn_list` in our `build_data` structure. Otherwise, it's identical.

## Random Builder - Take 1

Finally, we'll modify `random_builder` to use our `SimpleMapBuilder` with some new types to break out the creation steps:

```rust
pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    builder.start_with(SimpleMapBuilder::new());
    builder.with(RoomBasedSpawner::new());
    builder.with(RoomBasedStartingPosition::new());
    builder.with(RoomBasedStairs::new());
    builder
}
```

Notice that we're now taking a `RandomNumberGenerator` parameter. That's because we'd like to use the global RNG, rather than keep making new ones. This way, if the caller sets a "seed" - it will apply to world generation. This is intended to be the topic of a future chapter. We're also now returning a `BuilderChain` rather than a boxed trait - we're hiding the messy boxing/dynamic dispatch inside the implementation, so the caller doesn't have to worry about it. There's also two new types here: `RoomBasedSpawner` and `RoomBasedStartingPosition` - as well as a changed constructor for `SimpleMapBuilder` (it no longer accepts a depth parameter). We'll be looking at that in a second - but first, lets deal with the changes to the main program resulting from the new interface.

## Nice looking interface - but you broke stuff

We now have the *interface* we want - a good map of how the system interacts with the world. Unfortunately, the world is still expecting the setup we had before - so we need to fix it. In `main.rs`, we need to update our `generate_world_map` function to use the new interface:

```rust
fn generate_world_map(&mut self, new_depth : i32) {
    self.mapgen_index = 0;
    self.mapgen_timer = 0.0;
    self.mapgen_history.clear();
    let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
    let mut builder = map_builders::random_builder(new_depth, &mut rng);
    builder.build_map(&mut rng);
    std::mem::drop(rng);
    self.mapgen_history = builder.build_data.history.clone();
    let player_start;
    {
        let mut worldmap_resource = self.ecs.write_resource::<Map>();
        *worldmap_resource = builder.build_data.map.clone();
        player_start = builder.build_data.starting_position.as_mut().unwrap().clone();
    }

    // Spawn bad guys
    builder.spawn_entities(&mut self.ecs);
```

1. We reset `mapgen_index`, `mapgen_timer` and the `mapgen_history` so that the progress viewer will run from the beginning.
2. We obtain the RNG from the ECS `World`.
3. We create a new `random_builder` with the new interface, passing along the random number generator.
4. We tell it to build the new maps from the chain, also utilizing the RNG.
5. We call `std::mem::drop` on the RNG. This stops the "borrow" on it - so we're no longer borrowing `self` either. This prevents borrow-checker errors on the next phases of code.
6. We *clone* the map builder history into our own copy of the world's history. We copy it so we aren't destroying the builder, yet.
7. We set `player_start` to a *clone* of the builder's determined starting position. Note that we are calling `unwrap` - so the `Option` for a starting position *must* have a value at this point, or we'll crash. That's deliberate: we'd rather crash knowing that we forgot to set a starting point than have the program run in an unknown/confusing state.
8. We call `spawn_entities` to populate the map.

## Modifying SimpleMapBuilder

We can simplify `SimpleMapBuilder` (making it worthy of the name!) quite a bit. Here's the new code:

```rust
use super::{InitialMapBuilder, BuilderMap, Rect, apply_room_to_map, 
    apply_horizontal_tunnel, apply_vertical_tunnel };
use rltk::RandomNumberGenerator;

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.rooms_and_corridors(rng, build_data);
    }
}

impl SimpleMapBuilder {
        pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder{})
    }

    fn rooms_and_corridors(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;
        let mut rooms : Vec<Rect> = Vec::new();

        for i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width - w - 1) - 1;
            let y = rng.roll_dice(1, build_data.map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                apply_room_to_map(&mut build_data.map, &new_room);
                build_data.take_snapshot();

                if !rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = rooms[i as usize -1].center();
                    if rng.range(0,2) == 1 {
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(&mut build_data.map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(&mut build_data.map, prev_x, new_x, new_y);
                    }
                }

                rooms.push(new_room);
                build_data.take_snapshot();
            }
        }
        build_data.rooms = Some(rooms);
    }
}
```

This is basically the same as the old `SimpleMapBuilder`, but there's a number of changes:

* Notice that we're only applying the `InitialMapBuilder` trait - `MapBuilder` is no more.
* We're also not setting a starting position, or spawning entities - those are now the purview of other builders in the chain. We've basically distilled it down to just the room building algorithm.
* We set `build_data.rooms` to `Some(rooms)`. Not all algorithms support rooms - so our trait leaves the `Option` set to `None` until we fill it. Since the `SimpleMapBuilder` is all about rooms - we fill it in.

## Room-based spawning

Create a new file, `room_based_spawner.rs` in the `map_builders` directory. We're going to apply *just* the room populating system from the old `SimpleMapBuilder` here:

```rust
use super::{MetaMapBuilder, BuilderMap, spawner};
use rltk::RandomNumberGenerator;

pub struct RoomBasedSpawner {}

impl MetaMapBuilder for RoomBasedSpawner {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedSpawner {
        pub fn new() -> Box<RoomBasedSpawner> {
        Box::new(RoomBasedSpawner{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            for room in rooms.iter().skip(1) {
                spawner::spawn_room(&build_data.map, rng, room, build_data.map.depth, &mut build_data.spawn_list);
            }
        } else {
            panic!("Room Based Spawning only works after rooms have been created");
        }
    }
}
```

In this sub-module, we're implementing `MetaMapBuilder`: this builder requires that you already have a map. In `build`, we've copied the old room-based spawning code from `SimpleMapBuilder`, and modified it to operate on the builder's `rooms` structure. To that end, if we `if let` to obtain the inner value of the `Option`; if there isn't one, then we `panic!` and the program quits stating that room-based spawning is only going to work if you *have* rooms.

We've reduced the functionality to just one task: if there are rooms, we spawn monsters in them.

## Room-based starting position

This is very similar to room-based spawning, but places the player in the first room - just like it used to in `SimpleMapBuilder`. Create a new file, `room_based_starting_position` in `map_builders`:

```rust
use super::{MetaMapBuilder, BuilderMap, Position};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStartingPosition {}

impl MetaMapBuilder for RoomBasedStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedStartingPosition {
        pub fn new() -> Box<RoomBasedStartingPosition> {
        Box::new(RoomBasedStartingPosition{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let start_pos = rooms[0].center();
            build_data.starting_position = Some(Position{ x: start_pos.0, y: start_pos.1 });
        } else {
            panic!("Room Based Staring Position only works after rooms have been created");
        }
    }
}
```

## Room-based stairs

This is also very similar to how we generated exit stairs in `SimpleMapBuilder`. Make a new file, `room_based_stairs.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct RoomBasedStairs {}

impl MetaMapBuilder for RoomBasedStairs {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl RoomBasedStairs {
        pub fn new() -> Box<RoomBasedStairs> {
        Box::new(RoomBasedStairs{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        if let Some(rooms) = &build_data.rooms {
            let stairs_position = rooms[rooms.len()-1].center();
            let stairs_idx = build_data.map.xy_idx(stairs_position.0, stairs_position.1);
            build_data.map.tiles[stairs_idx] = TileType::DownStairs;
            build_data.take_snapshot();
        } else {
            panic!("Room Based Stairs only works after rooms have been created");
        }
    }
}
```

## Putting it together to make a simple map with the new framework

Let's take another look at `random_builder`:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(SimpleMapBuilder::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStartingPosition::new());
builder.with(RoomBasedStairs::new());
builder
```

Now that we've made all of the steps, this should make sense:

1. We *start with* a map generated with the `SimpleMapBuilder` generator.
2. We *modify* the map with the *meta-builder* `RoomBasedSpawner` to spawn entities in rooms.
3. We again *modify* the map with the *meta-builder* `RoomBasedStartingPosition` to start in the first room.
4. Once again, we *modify* the map with the *meta-builder* `RoomBasedStairs` to place a down staircase in the last room.

If you `cargo run` the project now, you'll let lots of warnings about unused code - but the game should play with just the simple map from our first section. You may be wondering *why* we've taken so much effort to keep things the same; hopefully, it will become clear as we clean up more builders!

## Cleaning up the BSP Dungeon Builder

Once again, we can seriously clean-up a map builder! Here's the new version of `bsp_dungeon.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, Map, Rect, apply_room_to_map, 
    TileType, draw_corridor};
use rltk::RandomNumberGenerator;

pub struct BspDungeonBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspDungeonBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BspDungeonBuilder {
        pub fn new() -> Box<BspDungeonBuilder> {
        Box::new(BspDungeonBuilder{
            rects: Vec::new(),
        })
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut rooms : Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects.push( Rect::new(2, 2, build_data.map.width-5, build_data.map.height-5) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(rng);
            let candidate = self.get_random_sub_rect(rect, rng);

            if self.is_possible(candidate, &build_data.map) {
                apply_room_to_map(&mut build_data.map, &candidate);
                rooms.push(candidate);
                self.add_subrects(rect);
                build_data.take_snapshot();
            }

            n_rooms += 1;
        }

        // Now we sort the rooms
        rooms.sort_by(|a,b| a.x1.cmp(&b.x1) );

        // Now we want corridors
        for i in 0..rooms.len()-1 {
            let room = rooms[i];
            let next_room = rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }
        build_data.rooms = Some(rooms);
    }

    fn add_subrects(&mut self, rect : Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects.push(Rect::new( rect.x1, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1, rect.y1 + half_height, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1 + half_height, half_width, half_height ));
    }

    fn get_random_rect(&mut self, rng : &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect : Rect, rng : &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;

        result.x1 += rng.roll_dice(1, 6)-1;
        result.y1 += rng.roll_dice(1, 6)-1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect : Rect, map : &Map) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        let mut can_build = true;

        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                if x > map.width-2 { can_build = false; }
                if y > map.height-2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                if can_build {
                    let idx = map.xy_idx(x, y);
                    if map.tiles[idx] != TileType::Wall { 
                        can_build = false; 
                    }
                }
            }
        }

        can_build
    }
}
```

Just like `SimpleMapBuilder`, we've stripped out all the non-room building code for a much cleaner piece of code. We're referencing the `build_data` struct from the builder, rather than making our own copies of everything - and the *meat* of the code is largely the same.

Now you can modify `random_builder` to make this map type:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(BspDungeonBuilder::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStartingPosition::new());
builder.with(RoomBasedStairs::new());
builder
```

If you `cargo run` now, you'll get a dungeon based on the `BspDungeonBuilder`. See how you are reusing the spawner, starting position and stairs code? That's definitely an improvement over the older versions - if you change one, it can now help on multiple builders!

## Same again for BSP Interior

Yet again, we can greatly clean up a builder - this time the `BspInteriorBuilder`. Here's the code for `bsp_interior.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, Rect, TileType, draw_corridor};
use rltk::RandomNumberGenerator;

const MIN_ROOM_SIZE : i32 = 8;

pub struct BspInteriorBuilder {
    rects: Vec<Rect>
}

impl InitialMapBuilder for BspInteriorBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BspInteriorBuilder {
        pub fn new() -> Box<BspInteriorBuilder> {
        Box::new(BspInteriorBuilder{
            rects: Vec::new()
        })
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut rooms : Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects.push( Rect::new(1, 1, build_data.map.width-2, build_data.map.height-2) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room, rng); // Divide the first room

        let rooms_copy = self.rects.clone();
        for r in rooms_copy.iter() {
            let room = *r;
            //room.x2 -= 1;
            //room.y2 -= 1;
            rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0 && idx < ((build_data.map.width * build_data.map.height)-1) as usize {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }

        // Now we want corridors
        for i in 0..rooms.len()-1 {
            let room = rooms[i];
            let next_room = rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }

        build_data.rooms = Some(rooms);
    }

    fn add_subrects(&mut self, rect : Rect, rng : &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }

        // Calculate boundaries
        let width  = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        let half_width = width / 2;
        let half_height = height / 2;

        let split = rng.roll_dice(1, 4);

        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new( rect.x1, rect.y1, half_width-1, height );
            self.rects.push( h1 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h1, rng); }
            let h2 = Rect::new( rect.x1 + half_width, rect.y1, half_width, height );
            self.rects.push( h2 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h2, rng); }
        } else {
            // Vertical split
            let v1 = Rect::new( rect.x1, rect.y1, width, half_height-1 );
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v1, rng); }
            let v2 = Rect::new( rect.x1, rect.y1 + half_height, width, half_height );
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v2, rng); }
        }
    }
}
```

You may test it by modifying `random_builder`:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(BspInteriorBuilder::new());
builder.with(RoomBasedSpawner::new());
builder.with(RoomBasedStartingPosition::new());
builder.with(RoomBasedStairs::new());
builder
```

`cargo run` will now take you around an interior builder.

## Cellular Automata

You should understand the basic idea here, now - we're breaking up builders into small chunks, and implementing the appropriate traits for the map type. Looking at Cellular Automata maps, you'll see that we do things a little differently:

* We make a map as usual. This obviously belongs in `CellularAutomataBuilder`.
* We search for a starting point close to the middle. This looks like it should be a separate step.
* We search the map for unreachable areas and cull them. This also looks like a separate step.
* We place the exit far from the starting position. That's *also* a different algorithm step.

The good news is that the last three of those are used in lots of other builders - so implementing them will let us reuse the code, and not keep repeating ourselves. The bad news is that if we run our cellular automata builder with the existing room-based steps, it will crash - we don't *have* rooms!

So we'll start by constructing the basic map builder. Like the others, this is mostly just rearranging code to fit with the new trait scheme. Here's the new `cellular_automata.rs` file:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomataBuilder {
        pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                if roll > 55 { build_data.map.tiles[idx] = TileType::Floor } 
                else { build_data.map.tiles[idx] = TileType::Wall }
            }
        }
        build_data.take_snapshot();

        // Now we iteratively apply cellular automata rules
        for _i in 0..15 {
            let mut newtiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height-1 {
                for x in 1..build_data.map.width-1 {
                    let idx = build_data.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if build_data.map.tiles[idx - 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + 1] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + build_data.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx - (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + (build_data.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if build_data.map.tiles[idx + (build_data.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    }
                    else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }
    }
}
```

### Non-Room Starting Points

It's entirely possible that we don't actually *want* to start in the middle of the map. Doing so presents lots of opportunities (and helps ensure connectivity), but maybe you would rather the player trudge through lots of map with less opportunity to pick the wrong direction. Maybe your story makes more sense if the player arrives at one end of the map and leaves via another. Lets implement a starting position system that takes a *preferred* starting point, and picks the closest valid tile. Create `area_starting_points.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, Position, TileType};
use rltk::RandomNumberGenerator;

pub enum XStart { LEFT, CENTER, RIGHT }

pub enum YStart { TOP, CENTER, BOTTOM }

pub struct AreaStartingPosition {
    x : XStart, 
    y : YStart
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl AreaStartingPosition {
        pub fn new(x : XStart, y : YStart) -> Box<AreaStartingPosition> {
        Box::new(AreaStartingPosition{
            x, y
        })
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let seed_x;
        let seed_y;

        match self.x {
            XStart::LEFT => seed_x = 1,
            XStart::CENTER => seed_x = build_data.map.width / 2,
            XStart::RIGHT => seed_x = build_data.map.width - 2
        }

        match self.y {
            YStart::TOP => seed_y = 1,
            YStart::CENTER => seed_y = build_data.map.height / 2,
            YStart::BOTTOM => seed_y = build_data.map.height - 2
        }

        let mut available_floors : Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if *tiletype == TileType::Floor {
                available_floors.push(
                    (
                        idx,
                        rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(idx as i32 % build_data.map.width, idx as i32 / build_data.map.width),
                            rltk::Point::new(seed_x, seed_y)
                        )
                    )
                );
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

        let start_x = available_floors[0].0 as i32 % build_data.map.width;
        let start_y = available_floors[0].0 as i32 / build_data.map.width;

        build_data.starting_position = Some(Position{x : start_x, y: start_y});
    }
}
```

We've covered the boilerplate enough to not need to go over it again - so lets step through the *build* function:

1. We are taking in a couple of `enum` types: preferred position on the X and Y axes.
2. So we set `seed_x` and `seed_y` to a point closest to the specified locations.
3. We iterate through the whole map, adding floor tiles to `available_floors` - and calculating the distance to the preferred starting point.
4. We sort the available tile list, so the lower distances are first.
5. We pick the first one on the list.

Note that we also `panic!` if there are no floors at all.

The great part here is that this will work for *any* map type - it searches for floors to stand on, and tries to find the closest starting point.

### Culling Unreachable Areas

We've previously had good luck with culling areas that can't be reached from the starting point. So lets formalize that into its own meta-builder. Create `cull_unreachable.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl CullUnreachable {
        pub fn new() -> Box<CullUnreachable> {
        Box::new(CullUnreachable{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x, 
            starting_pos.y
        );
        build_data.map.populate_blocked();
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(build_data.map.width as usize, build_data.map.height as usize, &map_starts , &build_data.map, 1000.0);
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == f32::MAX {
                    *tile = TileType::Wall;
                }
            }
        }
    }
}
```

You'll notice this is almost exactly the same as `remove_unreachable_areas_returning_most_distant` from `common.rs`, but without returning a Dijkstra map. That's the intent: we remove areas the player can't get to, and *only* do that.

### Voronoi-based spawning

We also need to replicate the functionality of Voronoi-based spawning. Create `voronoi_spawning.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType, spawner};
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

pub struct VoronoiSpawning {}

impl MetaMapBuilder for VoronoiSpawning {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl VoronoiSpawning {
        pub fn new() -> Box<VoronoiSpawning> {
        Box::new(VoronoiSpawning{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let mut noise_areas : HashMap<i32, Vec<usize>> = HashMap::new();
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1 .. build_data.map.height-1 {
            for x in 1 .. build_data.map.width-1 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if noise_areas.contains_key(&cell_value) {
                        noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }

        // Spawn the entities
        for area in noise_areas.iter() {
            spawner::spawn_region(&build_data.map, rng, area.1, build_data.map.depth, &mut build_data.spawn_list);
        }
    }
}
```

This is almost exactly the same as the code from `common.rs` we were calling in various builders, just modified to work within the builder chaining/builder map framework.

### Spawning a distant exit

Another commonly used piece of code generated a Dijkstra map of the level, starting at the player's entry point - and used that map to place the exit at the most distant location from the player. This was in `common.rs`, and we called it a lot. We'll turn this into a map building step; create `map_builders/distant_exit.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl DistantExit {
        pub fn new() -> Box<DistantExit> {
        Box::new(DistantExit{})
    }

    fn build(&mut self, _rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();
        let start_idx = build_data.map.xy_idx(
            starting_pos.x, 
            starting_pos.y
        );
        build_data.map.populate_blocked();
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(build_data.map.width as usize, build_data.map.height as usize, &map_starts , &build_data.map, 1000.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start != f32::MAX {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }

        // Place a staircase
        let stairs_idx = exit_tile.0;
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
```

Again, this is the same code we've used previously - just tweaked to match the new interface, so we won't go over it in detail.

### Testing Cellular Automata

We've finally got all the pieces together, so lets give it a test. In `random_builder`, we'll use the new builder chains:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(CellularAutomataBuilder::new());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

If you `cargo run` now, you'll get to play in a Cellular Automata generated map.

## Updating Drunkard's Walk

You should have a pretty good picture of what we're doing now, so we'll gloss over the changes to `drunkard.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType, Position, paint, Symmetry};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode { StartingPoint, Random }

pub struct DrunkardSettings {
    pub spawn_mode : DrunkSpawnMode,
    pub drunken_lifetime : i32,
    pub floor_percent: f32,
    pub brush_size: i32,
    pub symmetry: Symmetry
}

pub struct DrunkardsWalkBuilder {
    settings : DrunkardSettings
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
        pub fn new(settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder{
            settings
        }
    }

        pub fn open_area() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None
            }
        })
    }

        pub fn open_halls() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None
            },
        })
    }

        pub fn winding_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None
            },
        })
    }

        pub fn fat_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None
            },
        })
    }

        pub fn fearful_symmetry() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder{
            settings : DrunkardSettings{
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both
            },
        })
    }
    
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Set a central starting point
        let starting_position = Position{ x: build_data.map.width / 2, y: build_data.map.height / 2 };
        let start_idx = build_data.map.xy_idx(starting_position.x, starting_position.y);
        build_data.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        let mut digger_count = 0;
        while floor_tile_count  < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = starting_position.x;
                    drunk_y = starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = starting_position.x;
                        drunk_y = starting_position.y;
                    } else {
                        drunk_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    }
                }
            }
            let mut drunk_life = self.settings.drunken_lifetime;

            while drunk_life > 0 {
                let drunk_idx = build_data.map.xy_idx(drunk_x, drunk_y);
                if build_data.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(&mut build_data.map, self.settings.symmetry, self.settings.brush_size, drunk_x, drunk_y);
                build_data.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => { if drunk_x > 2 { drunk_x -= 1; } }
                    2 => { if drunk_x < build_data.map.width-2 { drunk_x += 1; } }
                    3 => { if drunk_y > 2 { drunk_y -=1; } }
                    _ => { if drunk_y < build_data.map.height-2 { drunk_y += 1; } }
                }

                drunk_life -= 1;
            }
            if did_something { 
                build_data.take_snapshot(); 
            }

            digger_count += 1;
            for t in build_data.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
    }
}
```

Once again, you can test it by adjusting `random_builder` to:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(DrunkardsWalkBuilder::fearful_symmetry());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

You can `cargo run` and see it in action.

## Update Diffusion-Limited Aggregation

This is more of the same, so we'll again just provide the code for `dla.rs`:

```rust
use super::{InitialMapBuilder, BuilderMap, TileType, Position, Symmetry, paint};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm { WalkInwards, WalkOutwards, CentralAttractor }

pub struct DLABuilder {
    algorithm : DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_percent: f32,
}


impl InitialMapBuilder for DLABuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DLABuilder {
        pub fn new() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

        pub fn walk_inwards() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

        pub fn walk_outwards() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

        pub fn central_attractor() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_percent: 0.25,
        })
    }

        pub fn insectoid() -> Box<DLABuilder> {
        Box::new(DLABuilder{
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_percent: 0.25,
        })
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Carve a starting seed
        let starting_position = Position{ x: build_data.map.width/2, y : build_data.map.height/2 };
        let start_idx = build_data.map.xy_idx(starting_position.x, starting_position.y);
        build_data.take_snapshot();
        build_data.map.tiles[start_idx] = TileType::Floor;
        build_data.map.tiles[start_idx-1] = TileType::Floor;
        build_data.map.tiles[start_idx+1] = TileType::Floor;
        build_data.map.tiles[start_idx-build_data.map.width as usize] = TileType::Floor;
        build_data.map.tiles[start_idx+build_data.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        while floor_tile_count  < desired_floor_tiles {

            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    while build_data.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < build_data.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < build_data.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, prev_x, prev_y);
                }

                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = starting_position.x;
                    let mut digger_y = starting_position.y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    while build_data.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < build_data.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < build_data.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, digger_x, digger_y);
                }

                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, build_data.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, build_data.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = build_data.map.xy_idx(digger_x, digger_y);

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham, 
                        rltk::Point::new( digger_x, digger_y ), 
                        rltk::Point::new( starting_position.x, starting_position.y )
                    );

                    while build_data.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = build_data.map.xy_idx(digger_x, digger_y);
                    }
                    paint(&mut build_data.map, self.symmetry, self.brush_size, prev_x, prev_y);
                }
            }

            build_data.take_snapshot();

            floor_tile_count = build_data.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }
    }    
}
```

## Updating the Maze Builder

Once again, here's the code for `maze.rs`:

```rust
use super::{Map,  InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

pub struct MazeBuilder {}

impl InitialMapBuilder for MazeBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MazeBuilder {
        pub fn new() -> Box<MazeBuilder> {
        Box::new(MazeBuilder{})
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Maze gen
        let mut maze = Grid::new((build_data.map.width / 2)-2, (build_data.map.height / 2)-2, rng);
        maze.generate_maze(build_data);
    }
}

/* Maze code taken under MIT from https://github.com/cyucelen/mazeGenerator/ */

const TOP : usize = 0;
const RIGHT : usize = 1;
const BOTTOM : usize = 2;
const LEFT : usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell{
            row,
            column,
            walls: [true, true, true, true],
            visited: false
        }
    }

    fn remove_walls(&mut self, next : &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        }
        else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        }
        else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        }
        else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng : &'a mut RandomNumberGenerator
}

impl<'a> Grid<'a> {
    fn new(width: i32, height:i32, rng: &mut RandomNumberGenerator) -> Grid {
        let mut grid = Grid{
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng
        };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column));
            }
        }

        grid
    }

    fn calculate_index(&self, row: i32, column: i32) -> i32 {
        if row < 0 || column < 0 || column > self.width-1 || row > self.height-1 {
            -1
        } else {
            column + (row * self.width)
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors : Vec<usize> = Vec::new();

        let current_row = self.cells[self.current].row;
        let current_column = self.cells[self.current].column;

        let neighbor_indices : [i32; 4] = [
            self.calculate_index(current_row -1, current_column),
            self.calculate_index(current_row, current_column + 1),
            self.calculate_index(current_row + 1, current_column),
            self.calculate_index(current_row, current_column - 1)
        ];

        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        if !neighbors.is_empty() {
            if neighbors.len() == 1 {
                return Some(neighbors[0]);
            } else {
                return Some(neighbors[(self.rng.roll_dice(1, neighbors.len() as i32)-1) as usize]);
            }
        }
        None
    }

    fn generate_maze(&mut self, build_data : &mut BuilderMap) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.cells[next].visited = true;
                    self.backtrace.push(self.current);
                    //   __lower_part__      __higher_part_
                    //   /            \      /            \
                    // --------cell1------ | cell2-----------
                    let (lower_part, higher_part) =
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if !self.backtrace.is_empty() {
                        self.current = self.backtrace[0];
                        self.backtrace.remove(0);
                    } else {
                        break;
                    }
                }
            }

            if i % 50 == 0 {
                self.copy_to_map(&mut build_data.map);
                build_data.take_snapshot();    
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map : &mut Map) {
        // Clear the map
        for i in map.tiles.iter_mut() { *i = TileType::Wall; }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] { map.tiles[idx - map.width as usize] = TileType::Floor }
            if !cell.walls[RIGHT] { map.tiles[idx + 1] = TileType::Floor }
            if !cell.walls[BOTTOM] { map.tiles[idx + map.width as usize] = TileType::Floor }
            if !cell.walls[LEFT] { map.tiles[idx - 1] = TileType::Floor }
        }
    }
}
```

## Updating Voronoi Maps

Here's the updated code for the Voronoi builder (in `voronoi.rs`):

```rust
use super::{InitialMapBuilder, BuilderMap, TileType};
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm { Pythagoras, Manhattan, Chebyshev }

pub struct VoronoiCellBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm
}


impl InitialMapBuilder for VoronoiCellBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder {
        pub fn new() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

        pub fn pythagoras() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

        pub fn manhattan() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder{
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        })
    }

    #[allow(clippy::map_entry)]
    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        // Make a Voronoi diagram. We'll do this the hard way to learn about the technique!
        let mut voronoi_seeds : Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, build_data.map.width-1);
            let vy = rng.roll_dice(1, build_data.map.height-1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32) ; self.n_seeds];
        let mut voronoi_membership : Vec<i32> = vec![0 ; build_data.map.width as usize * build_data.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % build_data.map.width;
            let y = i as i32 / build_data.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance;
                match self.distance_algorithm {           
                    DistanceAlgorithm::Pythagoras => {
                        distance = rltk::DistanceAlg::PythagorasSquared.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Manhattan => {
                        distance = rltk::DistanceAlg::Manhattan.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                    DistanceAlgorithm::Chebyshev => {
                        distance = rltk::DistanceAlg::Chebyshev.distance2d(
                            rltk::Point::new(x, y), 
                            pos.1
                        );
                    }
                }
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1..build_data.map.height-1 {
            for x in 1..build_data.map.width-1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[build_data.map.xy_idx(x-1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x+1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y-1)] != my_seed { neighbors += 1; }
                if voronoi_membership[build_data.map.xy_idx(x, y+1)] != my_seed { neighbors += 1; }

                if neighbors < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }    
}
```

## Updating Wave Function Collapse

Wave Function Collapse is a slightly different one to port, because it already had a concept of a "previous builder". That's gone now (chaining is automatic), so there's a bit more to update. Wave Function Collapse is a meta-builder, so it implements that trait, rather than the initial map builder. Overall, these changes make it a *lot* more simple! The changes all take place in `waveform_collapse/mod.rs`:

```rust
use super::{MetaMapBuilder, BuilderMap, Map, TileType};
use rltk::RandomNumberGenerator;
mod common;
use common::*;
mod constraints;
use constraints::*;
mod solver;
use solver::*;

/// Provides a map builder using the Wave Function Collapse algorithm.
pub struct WaveformCollapseBuilder {}

impl MetaMapBuilder for WaveformCollapseBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl WaveformCollapseBuilder {
    /// Constructor for waveform collapse.
        pub fn new() -> Box<WaveformCollapseBuilder> {
        Box::new(WaveformCollapseBuilder{})
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        const CHUNK_SIZE :i32 = 8;
        build_data.take_snapshot();

        let patterns = build_patterns(&build_data.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE, build_data);
                
        build_data.map = Map::new(build_data.map.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &build_data.map);
            while !solver.iteration(&mut build_data.map, rng) {
                build_data.take_snapshot();
            }
            build_data.take_snapshot();
            if solver.possible { break; } // If it has hit an impossible condition, try again
        }
        build_data.spawn_list.clear();
    }

    fn render_tile_gallery(&mut self, constraints: &[MapChunk], chunk_size: i32, build_data : &mut BuilderMap) {
        build_data.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            render_pattern_to_map(&mut build_data.map, &constraints[counter], chunk_size, x, y);

            x += chunk_size + 1;
            if x + chunk_size > build_data.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > build_data.map.height {
                    // Move to the next page
                    build_data.take_snapshot();
                    build_data.map = Map::new(0);

                    x = 1;
                    y = 1;
                }
            }

            counter += 1;
        }
        build_data.take_snapshot();
    }
}
```

You can test this with the following code:

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(VoronoiCellBuilder::pythagoras());
builder.with(WaveformCollapseBuilder::new());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(DistantExit::new());
builder
```

## Updating the Prefab Builder

So here's a fun one. The `PrefabBuilder` is both an `InitialMapBuilder` and a `MetaMapBuilder` - with shared code between the two. Fortunately, the traits are identical - so we can implement them both and call into the main `build` function from each! Rust is smart enough to figure out which one we're calling based on the trait we are storing - so `PrefabBuilder` can be placed in either an initial or a meta map builder.

The changes all take place in `prefab_builder/mod.rs`:

```rust
use super::{InitialMapBuilder, MetaMapBuilder, BuilderMap, TileType, Position};
use rltk::RandomNumberGenerator;
pub mod prefab_levels;
pub mod prefab_sections;
pub mod prefab_rooms;
use std::collections::HashSet;

#[derive(PartialEq, Copy, Clone)]
pub enum PrefabMode { 
    RexLevel{ template : &'static str },
    Constant{ level : prefab_levels::PrefabLevel },
    Sectional{ section : prefab_sections::PrefabSection },
    RoomVaults
}

pub struct PrefabBuilder {
    mode: PrefabMode
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap)  {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
        fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data : &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
        pub fn new() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::RoomVaults,
        })
    }

        pub fn rex_level(template : &'static str) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::RexLevel{ template },
        })
    }

        pub fn constant(level : prefab_levels::PrefabLevel) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::Constant{ level },
        })
    }

        pub fn sectional(section : prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::Sectional{ section },
        })
    }

        pub fn vaults() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder{
            mode : PrefabMode::RoomVaults,
        })
    }

    fn build(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel{template} => self.load_rex_map(&template, build_data),
            PrefabMode::Constant{level} => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional{section} => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data)
        }
        build_data.take_snapshot();    
    }

    fn char_to_map(&mut self, ch : char, idx: usize, build_data : &mut BuilderMap) {
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.starting_position = Some(Position{ x:x as i32, y:y as i32 });
            }
            '>' => build_data.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Health Potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
    }

        fn load_rex_map(&mut self, path: &str, build_data : &mut BuilderMap) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx, build_data);
                    }
                }
            }
        }
    }

    fn read_ascii_to_vec(template : &str) -> Vec<char> {
        let mut string_vec : Vec<char> = template.chars().filter(|a| *a != '\r' && *a !='\n').collect();
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' '; } }
        string_vec
    }

        fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data : &mut BuilderMap) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);
                    if i < string_vec.len() { self.char_to_map(string_vec[i], idx, build_data); }
                }
                i += 1;
            }
        }
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F, _rng: &mut RandomNumberGenerator, build_data : &mut BuilderMap)
        where F : FnMut(i32, i32) -> bool
    {
        let width = build_data.map.width;
        build_data.spawn_list.retain(|(idx, _name)| {
            let x = *idx as i32 % width;
            let y = *idx as i32 / width;
            filter(x, y)
        });
        build_data.take_snapshot();
    }

        fn apply_sectional(&mut self, section : &prefab_sections::PrefabSection, rng: &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template);
        
        // Place the new section
        let chunk_x;
        match section.placement.0 {
            HorizontalPlacement::Left => chunk_x = 0,
            HorizontalPlacement::Center => chunk_x = (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => chunk_x = (build_data.map.width-1) - section.width as i32
        }

        let chunk_y;
        match section.placement.1 {
            VerticalPlacement::Top => chunk_y = 0,
            VerticalPlacement::Center => chunk_y = (build_data.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => chunk_y = (build_data.map.height-1) - section.height as i32
        }

        // Build the map
        self.apply_previous_iteration(|x,y| {
            x < chunk_x || x > (chunk_x + section.width as i32) || y < chunk_y || y > (chunk_y + section.height as i32)
        }, rng, build_data);       

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx > 0 && tx < build_data.map.width as usize -1 && ty < build_data.map.height as usize -1 && ty > 0 {
                    let idx = build_data.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    if i < string_vec.len() { self.char_to_map(string_vec[i], idx, build_data); }
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng : &mut RandomNumberGenerator, build_data : &mut BuilderMap) {
        use prefab_rooms::*;

        // Apply the previous builder, and keep all entities it spawns (for now)
        self.apply_previous_iteration(|_x,_y| true, rng, build_data);

        // Do we want a vault at all?
        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
        if vault_roll < 4 { return; }

        // Note that this is a place-holder and will be moved out of this function
        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, CHECKERBOARD, SILLY_SMILE];

        // Filter the vault list down to ones that are applicable to the current depth
        let mut possible_vaults : Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| { build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth })
            .collect();

        if possible_vaults.is_empty() { return; } // Bail out if there's nothing to build

        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles : HashSet<usize> = HashSet::new();

        for _i in 0..n_vaults {

            let vault_index = if possible_vaults.len() == 1 { 0 } else { (rng.roll_dice(1, possible_vaults.len() as i32)-1) as usize };
            let vault = possible_vaults[vault_index];

            // We'll make a list of places in which the vault could fit
            let mut vault_positions : Vec<Position> = Vec::new();

            let mut idx = 0usize;
            loop {
                let x = (idx % build_data.map.width as usize) as i32;
                let y = (idx / build_data.map.width as usize) as i32;

                // Check that we won't overflow the map
                if x > 1 
                    && (x+vault.width as i32) < build_data.map.width-2
                    && y > 1 
                    && (y+vault.height as i32) < build_data.map.height-2
                {

                    let mut possible = true;
                    for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {

                            let idx = build_data.map.xy_idx(tx + x, ty + y);
                            if build_data.map.tiles[idx] != TileType::Floor {
                                possible = false;
                            }
                            if used_tiles.contains(&idx) {
                                possible = false;
                            }
                        }
                    }

                    if possible {
                        vault_positions.push(Position{ x,y });
                        break;
                    }

                }

                idx += 1;
                if idx >= build_data.map.tiles.len()-1 { break; }
            }

            if !vault_positions.is_empty() {
                let pos_idx = if vault_positions.len()==1 { 0 } else { (rng.roll_dice(1, vault_positions.len() as i32)-1) as usize };
                let pos = &vault_positions[pos_idx];

                let chunk_x = pos.x;
                let chunk_y = pos.y;

                let width = build_data.map.width; // The borrow checker really doesn't like it
                let height = build_data.map.height; // when we access `self` inside the `retain`
                build_data.spawn_list.retain(|e| {
                    let idx = e.0 as i32;
                    let x = idx % width;
                    let y = idx / height;
                    x < chunk_x || x > chunk_x + vault.width as i32 || y < chunk_y || y > chunk_y + vault.height as i32
                });

                let string_vec = PrefabBuilder::read_ascii_to_vec(vault.template);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        let idx = build_data.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                        if i < string_vec.len() { self.char_to_map(string_vec[i], idx, build_data); }
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                build_data.take_snapshot();

                possible_vaults.remove(vault_index);
            }
        }
    }
}
```

You can test our recent changes with the following code in `random_builder` (in `map_builders/mod.rs`):

```rust
let mut builder = BuilderChain::new(new_depth);
builder.start_with(VoronoiCellBuilder::pythagoras());
builder.with(WaveformCollapseBuilder::new());
builder.with(PrefabBuilder::vaults());
builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
builder.with(CullUnreachable::new());
builder.with(VoronoiSpawning::new());
builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
builder.with(DistantExit::new());
builder
```

This demonstrates the power of our approach - we're putting a lot of functionality together from small building blocks. In this example we are:

1. *Starting* with a map generated with the `VoronoiBuilder` in `Pythagoras` mode.
2. *Modifying* the map with a `WaveformCollapseBuilder` run, which will rearrange the map like a jigsaw puzzle.
3. *Modifying* the map by placing vaults, via the `PrefabBuilder` (in Vaults mode).
4. *Modifying* the map with `AreaStartingPositions` indicating that we'd like to start near the middle of the map.
5. *Modifying* the map to cull unreachable areas.
6. *Modifying* the map to spawn entities using a Voronoi spawning method.
7. *Modifying* the map to add an underground fortress, again using the `PrefabBuilder`.
8. *Modifying* the map to add an exit staircase, in the most distant location.

## Delete the MapBuilder Trait and bits from common

Now that we have the builder mechanism in place, there's some old code we can delete. From `common.rs`, we can delete `remove_unreachable_areas_returning_most_distant` and `generate_voronoi_spawn_regions`; we've replaced them with builder steps.

We can also open `map_builders/mod.rs` and delete the `MapBuilder` trait and its implementation: we've completely replaced it now.

## Randomize

As usual, we'd like to go back to having map generation be random. We're going to break the process up into two steps. We'll make a new function, `random_initial_builder` that rolls a dice and picks the *starting* builder. It also returns a `bool`, indicating whether or not we picked an algorithm that provides room data. The basic function should look familiar, but we've got rid of all the `Box::new` calls - the constructors make boxes for us, now:

```rust
fn random_initial_builder(rng: &mut rltk::RandomNumberGenerator) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 17);
    let result : (Box<dyn InitialMapBuilder>, bool);
    match builder {
        1 => result = (BspDungeonBuilder::new(), true),
        2 => result = (BspInteriorBuilder::new(), true),
        3 => result = (CellularAutomataBuilder::new(), false),
        4 => result = (DrunkardsWalkBuilder::open_area(), false),
        5 => result = (DrunkardsWalkBuilder::open_halls(), false),
        6 => result = (DrunkardsWalkBuilder::winding_passages(), false),
        7 => result = (DrunkardsWalkBuilder::fat_passages(), false),
        8 => result = (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => result = (MazeBuilder::new(), false),
        10 => result = (DLABuilder::walk_inwards(), false),
        11 => result = (DLABuilder::walk_outwards(), false),
        12 => result = (DLABuilder::central_attractor(), false),
        13 => result = (DLABuilder::insectoid(), false),
        14 => result = (VoronoiCellBuilder::pythagoras(), false),
        15 => result = (VoronoiCellBuilder::manhattan(), false),
        16 => result = (PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED), false),
        _ => result = (SimpleMapBuilder::new(), true)
    }
    result
}
```

This is a pretty straightforward function - we roll a dice, match on the result table and return the builder and room information we picked. Now we'll modify our `random_builder` function to use it:

```rust
pub fn random_builder(new_depth: i32, rng: &mut rltk::RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth);
    let (random_starter, has_rooms) = random_initial_builder(rng);
    builder.start_with(random_starter);
    if has_rooms {
        builder.with(RoomBasedSpawner::new());
        builder.with(RoomBasedStairs::new());
        builder.with(RoomBasedStartingPosition::new());
    } else {
        builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
        builder.with(CullUnreachable::new());
        builder.with(VoronoiSpawning::new());
        builder.with(DistantExit::new());
    }

    if rng.roll_dice(1, 3)==1 {
        builder.with(WaveformCollapseBuilder::new());
    }

    if rng.roll_dice(1, 20)==1 {
        builder.with(PrefabBuilder::sectional(prefab_builder::prefab_sections::UNDERGROUND_FORT));
    }

    builder.with(PrefabBuilder::vaults());

    builder
}
```

This should look familiar. This function:

1. Selects a random room using the function we just created.
2. If the builder provides room data, we chain `RoomBasedSpawner`, `RoomBasedStairs` and `RoomBasedStartingPositiosn` - the three important steps required for room data.
3. If the builder *doesn't* provide room information, we chain `AreaStartingPosition`, `CullUnreachable`, `VoronoiSpawning` and `DistantExit` - the defaults we used to apply inside each builder.
4. We roll a 3-sided die; if it comes up 1 - we apply a `WaveformCollapseBuilder` to rearrange the map.
5. We roll a 20-sided die; if it comes up 1 - we apply our Underground Fort prefab.
6. We apply vault creation to the final map, giving a chance for pre-made rooms to appear.

## Wrap-Up

This has been an *enormous* chapter, but we've accomplished a lot:

* We now have a consistent builder interface for chaining as many meta-map modifiers as we want to our build chain. This should let us build the maps we want.
* Each builder now does *just one task* - so it's much more obvious where to go if you need to fix/debug them.
* Builders are no longer responsible for making other builders - so we've culled a swathe of code and moved the opportunity for bugs to creep in to just one (simple) control flow.

This sets the stage for the next chapter, which will look at more ways to use filters to modify your map.

...

**The source code for this chapter may be found [here](https://github.com/thebracket/rustrogueliketutorial/tree/master/chapter-36-layers)**

[Run this chapter's example with web assembly, in your browser (WebGL2 required)](https://bfnightly.bracketproductions.com/rustbook/wasm/chapter-36-layers/)

---

Copyright (C) 2019, Herbert Wolverson.

---
