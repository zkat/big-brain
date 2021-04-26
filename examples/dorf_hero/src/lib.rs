use bevy::{
    asset::LoadState,
    prelude::*,
    sprite::{TextureAtlas, TextureAtlasBuilder},
    window::WindowMode,
};
use bevy_tilemap::prelude::*;
use big_brain::prelude::*;
use rand::Rng;

use ai::DorfHeroAiPlugin;
use components::{EvilDorf, Hero, Position, Render};
use resources::{GameState, TileSpriteHandles};
use systems::DorfHeroSystemsPlugin;

mod ai;
mod components;
mod resources;
mod systems;
mod util;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    Setup,
    Simulating,
}

pub fn start() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Dorf Hero".to_string(),
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
        .add_plugin(DorfHeroAiPlugin)
        .add_plugin(DorfHeroSystemsPlugin(AppState::Simulating))
        .add_state(AppState::Setup)
        .add_system_set(SystemSet::on_enter(AppState::Setup).with_system(setup.system()))
        .add_system_set(SystemSet::on_update(AppState::Setup).with_system(check_loaded.system()))
        .add_system_set(SystemSet::on_exit(AppState::Setup).with_system(load.system()))
        .add_system_set(
            SystemSet::on_enter(AppState::Simulating).with_system(build_random_dungeon.system()),
        )
        .run()
}

// Here we have a pretty typical bundle, except we've added a ThinkerBuilder to it.
#[derive(Bundle)]
struct PlayerBundle {
    player: Hero,
    position: Position,
    render: Render,
    thinker: ThinkerBuilder,
}

#[derive(Bundle)]
struct EvilBundle {
    evil: EvilDorf,
    position: Position,
    render: Render,
    thinker: ThinkerBuilder,
}

const CHUNK_WIDTH: u32 = 16;
const CHUNK_HEIGHT: u32 = 16;
const TILEMAP_WIDTH: i32 = CHUNK_WIDTH as i32 * 40;
const TILEMAP_HEIGHT: i32 = CHUNK_HEIGHT as i32 * 40;

fn setup(mut tile_sprite_handles: ResMut<TileSpriteHandles>, asset_server: Res<AssetServer>) {
    tile_sprite_handles.handles = asset_server.load_folder("textures").unwrap();
}

fn load(
    mut commands: Commands,
    sprite_handles: Res<TileSpriteHandles>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
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
        .insert(Timer::from_seconds(0.1, true));
}

fn build_random_dungeon(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut query: Query<&mut Tilemap>,
) {
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
            game_state.walls.insert(tile_a);
            game_state.walls.insert(tile_b);
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
            game_state.walls.insert(tile_a);
            game_state.walls.insert(tile_b);
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
                game_state.walls.insert((x, y));
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
            player: Hero,
            position: Position { x: 0, y: 0 },
            render: Render {
                sprite_index: dwarf_sprite_index,
                sprite_order: 1,
            },
            thinker: Thinker::build()
                .picker(FirstToScore::new(80.))
                .otherwise(ai::actions::meander::Meander::build()),
        });

        let evil_sprite: Handle<Texture> =
            asset_server.get_handle("textures/square-evil-dwarf.png");
        let evil_sprite_index = texture_atlas.get_texture_index(&evil_sprite).unwrap();
        // We add in a Z order of 1 to place the tile above the background on Z
        // order 0.
        let evil_tile = Tile {
            point: (0, 0),
            sprite_order: 1,
            sprite_index: evil_sprite_index,
            ..Default::default()
        };
        tiles.push(evil_tile);

        commands.spawn().insert_bundle(EvilBundle {
            evil: EvilDorf,
            position: Position { x: 4, y: 8 },
            render: Render {
                sprite_index: evil_sprite_index,
                sprite_order: 1,
            },
            thinker: Thinker::build()
                .picker(FirstToScore::new(80.))
                .otherwise(ai::actions::meander::Meander::build()),
        });

        // Now we pass all the tiles to our map.
        map.insert_tiles(tiles).unwrap();
    }
}

fn check_loaded(
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
    sprite_handles: Res<TileSpriteHandles>,
) {
    if let LoadState::Loaded =
        asset_server.get_group_load_state(sprite_handles.handles.iter().map(|handle| handle.id))
    {
        state
            .set(AppState::Simulating)
            .expect("Failed to switch AppState");
    }
}
