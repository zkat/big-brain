/*!
This is a more involved big-brain example. In this example, we're going to define an AI "Player" which will autonomously explore a randomized dungeon. They will also react to various situations they come across, such as enemies, power-ups, and their own personal needs (like going to the bathroom).
*/

use bevy::{
    asset::LoadState,
    prelude::*,
    render::camera::Camera,
    sprite::{TextureAtlas, TextureAtlasBuilder},
    utils::HashSet,
    window::WindowMode,
};
use bevy_tilemap::prelude::*;
use big_brain::prelude::*;
use rand::Rng;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Endless Dungeon".to_string(),
            width: 1024.,
            height: 1024.,
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        .init_resource::<TileSpriteHandles>()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilemapDefaultPlugins)
        // Here's the first important thing for you to do: include the plugin!
        // If big-brain doesn't seem to be working at all, this might be what
        // you're missing!
        .add_plugin(BigBrainPlugin)
        .add_startup_system(setup.system())
        .add_system(load.system())
        .add_system(build_random_dungeon.system())
        .add_system(meander_action.system())
        .run()
}

// Here we have a pretty typical bundle, except we've added a ThinkerBuilder to it.
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    position: Position,
    render: Render,
    thinker: ThinkerBuilder,
}

// Let's define our "default" action, which will be used whenever there's nothing in particular getting our dorf's attention.
#[derive(Default, Debug, Clone)]
struct MeanderBuilder;

#[derive(Debug, Default, Clone)]
struct Meander {
    dx: i32,
    dy: i32,
}

impl Meander {
    fn build() -> MeanderBuilder {
        MeanderBuilder
    }
}

impl ActionBuilder for MeanderBuilder {
    fn build(&self, cmd: &mut Commands, action: Entity, _actor: Entity) {
        cmd.entity(action).insert(Meander::default());
    }
}

fn meander_action(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut map_query: Query<(&mut Tilemap, &mut Timer)>,
    mut player_query: Query<(&mut Position, &Render), With<Player>>,
    mut action_q: Query<(&mut Meander, &Actor, &mut ActionState)>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
) {
    if !game_state.map_loaded {
        return;
    }

    for (mut map, mut timer) in map_query.iter_mut() {
        timer.tick(time.delta());
        if !timer.finished() {
            continue;
        }
        for (mut meander, Actor(actor), mut state) in action_q.iter_mut() {
            match *state {
                ActionState::Cancelled => {
                    // *ALWAYS* handle Cancelled, even if you're doing a single-tick action. Your action might end up hanging otherwise.
                    *state = ActionState::Success;
                    continue;
                }
                ActionState::Init | ActionState::Success | ActionState::Failure => {
                    continue;
                }
                // These two fall through to logic :)
                ActionState::Requested => {
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {}
            }
            if let Ok((mut pos, render)) = player_query.get_mut(*actor) {
                for (_camera, mut camera_transform) in camera_query.iter_mut() {
                    let previous_position = *pos;
                    if (meander.dx == 0 && meander.dy == 0)
                        || !game_state.try_move_player(
                            &mut pos,
                            &mut camera_transform.translation,
                            (meander.dx, meander.dy),
                        )
                    {
                        let mut rng = rand::thread_rng();
                        meander.dx = rng.gen_range(-1..=1);
                        meander.dy = rng.gen_range(-1..=1);
                    } else {
                        move_sprite(&mut map, previous_position, *pos, render);
                    }
                }
            }
        }
    }
}

const CHUNK_WIDTH: u32 = 16;
const CHUNK_HEIGHT: u32 = 16;
const TILEMAP_WIDTH: i32 = CHUNK_WIDTH as i32 * 40;
const TILEMAP_HEIGHT: i32 = CHUNK_HEIGHT as i32 * 40;

#[derive(Default, Clone)]
struct TileSpriteHandles {
    handles: Vec<HandleUntyped>,
    atlas_loaded: bool,
}

#[derive(Default, Copy, Clone, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Default)]
struct Render {
    sprite_index: usize,
    sprite_order: usize,
}

#[derive(Default)]
struct Player {}

#[derive(Default, Clone)]
struct GameState {
    map_loaded: bool,
    spawned: bool,
    collisions: HashSet<(i32, i32)>,
}

impl GameState {
    fn try_move_player(
        &mut self,
        position: &mut Position,
        camera_translation: &mut Vec3,
        delta_xy: (i32, i32),
    ) -> bool {
        let new_pos = (position.x + delta_xy.0, position.y + delta_xy.1);
        if !self.collisions.contains(&new_pos) {
            position.x += delta_xy.0;
            position.y += delta_xy.1;
            camera_translation.x += delta_xy.0 as f32 * 32.;
            camera_translation.y += delta_xy.1 as f32 * 32.;
            true
        } else {
            false
        }
    }
}

fn move_sprite(
    map: &mut Tilemap,
    previous_position: Position,
    position: Position,
    render: &Render,
) {
    // We need to first remove where we were prior.
    map.clear_tile((previous_position.x, previous_position.y), 1)
        .unwrap();
    // We then need to update where we are going!
    let tile = Tile {
        point: (position.x, position.y),
        sprite_index: render.sprite_index,
        sprite_order: render.sprite_order,
        ..Default::default()
    };
    map.insert_tile(tile).unwrap();
}

fn setup(mut tile_sprite_handles: ResMut<TileSpriteHandles>, asset_server: Res<AssetServer>) {
    tile_sprite_handles.handles = asset_server.load_folder("textures").unwrap();
}

fn load(
    mut commands: Commands,
    mut sprite_handles: ResMut<TileSpriteHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
    asset_server: Res<AssetServer>,
) {
    if sprite_handles.atlas_loaded {
        return;
    }

    // Lets load all our textures from our folder!
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        for handle in sprite_handles.handles.iter() {
            let texture = textures.get(handle).unwrap();
            texture_atlas_builder.add_texture(handle.clone_weak().typed::<Texture>(), &texture);
        }

        let texture_atlas = texture_atlas_builder.finish(&mut textures).unwrap();
        let atlas_handle = texture_atlases.add(texture_atlas);

        // These are fairly advanced configurations just to quickly showcase
        // them.
        let tilemap = Tilemap::builder()
            .dimensions(TILEMAP_WIDTH as u32, TILEMAP_HEIGHT as u32)
            .chunk_dimensions(CHUNK_WIDTH, CHUNK_HEIGHT, 1)
            .texture_dimensions(32, 32)
            .auto_chunk()
            .auto_spawn(2, 2)
            .add_layer(
                TilemapLayer {
                    kind: LayerKind::Dense,
                },
                0,
            )
            .texture_atlas(atlas_handle)
            .finish()
            .unwrap();

        let tilemap_components = TilemapBundle {
            tilemap,
            visible: Visible {
                is_visible: true,
                is_transparent: true,
            },
            transform: Default::default(),
            global_transform: Default::default(),
        };
        commands
            .spawn()
            .insert_bundle(OrthographicCameraBundle::new_2d());
        commands
            .spawn()
            .insert_bundle(tilemap_components)
            .insert(Timer::from_seconds(0.075, true));

        sprite_handles.atlas_loaded = true;
    }
}

fn build_random_dungeon(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Tilemap>,
) {
    if game_state.map_loaded {
        return;
    }

    for mut map in query.iter_mut() {
        // Then we need to find out what the handles were to our textures we are going to use.
        let floor_sprite: Handle<Texture> = asset_server.get_handle("textures/square-floor.png");
        let wall_sprite: Handle<Texture> = asset_server.get_handle("textures/square-wall.png");
        let texture_atlas = texture_atlases.get(map.texture_atlas()).unwrap();
        let floor_idx = texture_atlas.get_texture_index(&floor_sprite).unwrap();
        let wall_idx = texture_atlas.get_texture_index(&wall_sprite).unwrap();

        // Now we fill the entire space with floors.
        let mut tiles = Vec::new();
        for y in 0..TILEMAP_HEIGHT {
            for x in 0..TILEMAP_WIDTH {
                let y = y - TILEMAP_HEIGHT / 2;
                let x = x - TILEMAP_WIDTH / 2;
                // By default tile sets the Z order at 0. Lower means that tile
                // will render lower than others. 0 is the absolute bottom
                // level which is perfect for backgrounds.
                let tile = Tile {
                    point: (x, y),
                    sprite_index: floor_idx,
                    ..Default::default()
                };
                tiles.push(tile);
            }
        }

        for x in 0..TILEMAP_WIDTH {
            let x = x - TILEMAP_WIDTH / 2;
            let tile_a = (x, -TILEMAP_HEIGHT / 2);
            let tile_b = (x, TILEMAP_HEIGHT / 2 - 1);
            tiles.push(Tile {
                point: tile_a,
                sprite_index: wall_idx,
                ..Default::default()
            });
            tiles.push(Tile {
                point: tile_b,
                sprite_index: wall_idx,
                ..Default::default()
            });
            game_state.collisions.insert(tile_a);
            game_state.collisions.insert(tile_b);
        }

        // Then the wall tiles on the Y axis.
        for y in 0..TILEMAP_HEIGHT {
            let y = y - TILEMAP_HEIGHT / 2;
            let tile_a = (-TILEMAP_WIDTH / 2, y);
            let tile_b = (TILEMAP_WIDTH / 2 - 1, y);
            tiles.push(Tile {
                point: tile_a,
                sprite_index: wall_idx,
                ..Default::default()
            });
            tiles.push(Tile {
                point: tile_b,
                sprite_index: wall_idx,
                ..Default::default()
            });
            game_state.collisions.insert(tile_a);
            game_state.collisions.insert(tile_b);
        }

        // Lets just generate some random walls to sparsely place around the dungeon!
        let range = (TILEMAP_WIDTH * TILEMAP_HEIGHT) as usize / 5;
        let mut rng = rand::thread_rng();
        for _ in 0..range {
            let x = rng.gen_range((-TILEMAP_WIDTH / 2)..(TILEMAP_WIDTH / 2));
            let y = rng.gen_range((-TILEMAP_HEIGHT / 2)..(TILEMAP_HEIGHT / 2));
            let coord = (x, y, 0i32);
            if coord != (0, 0, 0) {
                tiles.push(Tile {
                    point: (x, y),
                    sprite_index: wall_idx,
                    ..Default::default()
                });
                game_state.collisions.insert((x, y));
            }
        }

        // The above should give us a neat little randomized dungeon! However,
        // we are missing a hero! First, we need to add a layer. We must make
        // this layer `Sparse` else we will lose efficiency with our data!
        //
        // You might've noticed that we didn't create a layer for z_layer 0 but
        // yet it still works and exists. By default if a layer doesn't exist
        // and tiles need to be written there then a Dense layer is created
        // automatically.
        map.add_layer(
            TilemapLayer {
                kind: LayerKind::Sparse,
            },
            1,
        )
        .unwrap();

        // Now lets add in a dwarf friend!
        let dwarf_sprite: Handle<Texture> = asset_server.get_handle("textures/square-dwarf.png");
        let dwarf_sprite_index = texture_atlas.get_texture_index(&dwarf_sprite).unwrap();
        // We add in a Z order of 1 to place the tile above the background on Z
        // order 0.
        let dwarf_tile = Tile {
            point: (0, 0),
            sprite_order: 1,
            sprite_index: dwarf_sprite_index,
            ..Default::default()
        };
        tiles.push(dwarf_tile);

        commands.spawn().insert_bundle(PlayerBundle {
            player: Player {},
            position: Position { x: 0, y: 0 },
            render: Render {
                sprite_index: dwarf_sprite_index,
                sprite_order: 1,
            },
            thinker: Thinker::build()
                .picker(FirstToScore::new(80.))
                .otherwise(Meander::build()),
        });

        // Now we pass all the tiles to our map.
        map.insert_tiles(tiles).unwrap();

        game_state.map_loaded = true;
    }
}
