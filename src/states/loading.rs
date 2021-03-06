use crate::{
    audio::initialise_audio,
    components::{graphics::SpriteSheetType, physics::PlatformCuboid},
    level::Level,
    states::{
        load_level::LoadLevelState,
        pizzatopia::{MyEvents, Pizzatopia},
    },
    ui::{
        file_picker::{FilePickerFilename, DIR_LEVELS},
        UiStack,
    },
};
use amethyst::{
    assets::{
        Asset, AssetStorage, Completion, Format, Handle, Loader, Prefab, PrefabData, PrefabLoader,
        PrefabLoaderSystemDesc, ProcessingState, Processor, Progress, ProgressCounter, RonFormat,
    },
    core::{
        bundle::SystemBundle,
        ecs::{Read, SystemData, World},
        frame_limiter::{FrameRateLimitStrategy, *},
        shrev::{EventChannel, ReaderId},
        transform::Transform,
        EventReader, SystemDesc, Time,
    },
    derive::EventReader,
    ecs::prelude::{Component, DenseVecStorage, Dispatcher, DispatcherBuilder, Entity},
    input::{is_key_down, InputHandler, StringBindings, VirtualKeyCode},
    prelude::*,
    renderer::{
        debug_drawing::{DebugLines, DebugLinesComponent, DebugLinesParams},
        rendy::{
            hal::image::{Filter, SamplerInfo, WrapMode},
            texture::image::{ImageTextureConfig, Repr, TextureKind},
        },
        Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture,
    },
    ui::{FontAsset, RenderUi, TtfFormat, UiBundle, UiCreator, UiEvent, UiFinder, UiText},
    utils::{
        application_root_dir,
        fps_counter::{FpsCounter, FpsCounterBundle},
    },
    winit::Event,
};
use bami::Input;
use log::warn;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

pub struct AssetsDir(pub PathBuf);

#[derive(Default)]
pub struct DrawDebugLines(pub bool);

pub struct LoadingState {
    /// Tracks loaded assets.
    progress_counter: ProgressCounter,
}

impl Default for LoadingState {
    fn default() -> Self {
        LoadingState {
            progress_counter: ProgressCounter::default(),
        }
    }
}

impl<'s> State<GameData<'s, 's>, MyEvents> for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData<'s, 's>>) {
        initialise_audio(data.world);
        let platform_size_prefab_handle =
            data.world.exec(|loader: PrefabLoader<'_, PlatformCuboid>| {
                loader.load("prefab/tile_size.ron", RonFormat, ())
            });
        data.world.insert(platform_size_prefab_handle.clone());

        data.world.insert(DebugLines::new());
        data.world.insert(DebugLinesParams { line_width: 2.0 });
        data.world.insert(DrawDebugLines(false));

        data.world
            .insert(AssetsDir(application_root_dir().unwrap().join("assets")));
        let font_handle: Handle<FontAsset> = data.world.read_resource::<Loader>().load(
            "font/LibreBaskerville-Bold.ttf",
            TtfFormat,
            &mut self.progress_counter,
            &data.world.read_resource(),
        );
        data.world.insert(font_handle);
        data.world
            .insert(BTreeMap::<u8, Handle<SpriteSheet>>::new());

        self.add_new_sprite_sheet(data.world, "texture/tiles", SpriteSheetType::Tiles as u8);
        self.add_new_sprite_sheet(data.world, "texture/walk", SpriteSheetType::Didi as u8);
        self.add_new_sprite_sheet(
            data.world,
            "texture/rolling_hills_bg",
            SpriteSheetType::RollingHillsBg as u8,
        );
        self.add_new_sprite_sheet(
            data.world,
            "texture/spritesheet2",
            SpriteSheetType::Snap as u8,
        );
        self.add_new_sprite_sheet(data.world, "texture/ui", SpriteSheetType::Ui as u8);
        self.add_new_sprite_sheet(
            data.world,
            "texture/animation",
            SpriteSheetType::Animation as u8,
        );

        data.world.insert(FilePickerFilename::new(
            "level0.ron".to_string(),
            "level0.ron".to_string(),
        ));
        data.world.insert(UiStack::default());
    }

    fn update(
        &mut self,
        mut data: StateData<'_, GameData<'s, 's>>,
    ) -> Trans<GameData<'s, 's>, MyEvents> {
        data.data.update(&mut data.world);
        if self.progress_counter.is_complete() {
            return Trans::Switch(Box::new(LoadLevelState::default()));
        } else {
            Trans::None
        }
    }
}

fn load_spritesheet(
    filename_without_extension: String,
    world: &mut World,
    progress: &mut ProgressCounter,
) -> Handle<SpriteSheet> {
    // Load the sprite sheet necessary to render the graphics.
    // The texture is the pixel data
    // `texture_handle` is a cloneable reference to the texture
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            filename_without_extension.clone() + ".png",
            ImageFormat(get_image_texure_config()),
            progress,
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        filename_without_extension.clone() + ".ron", // Here we load the associated ron file
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

pub fn get_image_texure_config() -> ImageTextureConfig {
    ImageTextureConfig {
        // Determine format automatically
        format: None,
        // Color channel
        repr: Repr::Srgb,
        // Two-dimensional texture
        kind: TextureKind::D2,
        sampler_info: SamplerInfo::new(Filter::Nearest, WrapMode::Clamp),
        // Don't generate mipmaps for this image
        generate_mips: false,
        premultiply_alpha: true,
    }
}

impl LoadingState {
    fn add_new_sprite_sheet(&mut self, world: &mut World, name: &str, sheet_number: u8) {
        let name = String::from(name);
        let sprites = load_spritesheet(name.clone(), world, &mut self.progress_counter);
        world
            .write_resource::<BTreeMap<u8, Handle<SpriteSheet>>>()
            .insert(sheet_number, sprites);
    }
}
