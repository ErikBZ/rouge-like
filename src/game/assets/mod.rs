use bevy::prelude::*;
use bevy::log::info;
use bevy_asset::{io::Reader, Asset, AssetLoader, LoadContext};
use crate::game::units::UnitStats;
use crate::game::weapon::Weapon;
use std::marker::PhantomData;
use ron::de::from_bytes;
use serde::{Deserialize, Serialize};
use thiserror::Error;


#[derive(Asset, Debug, TypePath, Deserialize, Serialize)]
pub struct UnitAsset {
    pub units: Vec<UnitStats>
}

#[derive(Asset, Debug, TypePath, Deserialize)]
pub struct WeaponAsset {
    units: Vec<Weapon>
}

// Credit: Used a lot of code from bevy_common_assets. Thanks [https://github.com/NiklasEi/bevy_common_assets.git]
// Interesting, for a struct with a generic param, you need to use it right away
// so that why's you need PhantomData
pub struct GameAssetLoader<A> {
    extensions: Vec<&'static str>,
    _marker: PhantomData<A>
}

/// Plugin to load your asset type `A` from ron files.
pub struct GameAssetPlugin;

impl Plugin for GameAssetPlugin
{
    fn build(&self, app: &mut App) {
        app
            .init_asset::<WeaponAsset>()
            .init_asset::<UnitAsset>()
            .register_asset_loader(GameAssetLoader::<UnitAsset> {
                extensions: vec!["units.ron"],
                _marker: PhantomData
            })
            .register_asset_loader(GameAssetLoader::<WeaponAsset> {
                extensions: vec!["weapons.ron"],
                _marker: PhantomData
            });
    }
}

/// Possible errors that can be produced by [`RonAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RonLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON Error](serde_ron::error::SpannedError)
    #[error("Could not parse RON: {0}")]
    RonError(#[from] ron::error::SpannedError),
}

impl<A> AssetLoader for GameAssetLoader<A>
where
    for<'de> A: serde::Deserialize<'de> + Asset,
{
    type Asset = A;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loaded the loader");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = from_bytes::<A>(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}
