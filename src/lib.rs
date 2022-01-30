use core::convert::Infallible;
use std::{borrow::Cow, fmt::Debug, marker::PhantomData, ops::Deref};

use rkyv::{
    archived_root,
    de::deserializers::{SharedDeserializeMap, SharedDeserializeMapError},
    from_bytes_unchecked,
    ser::serializers::{
        AllocScratchError, AllocSerializer, CompositeSerializerError, SharedSerializeMapError,
    },
    to_bytes, Archive, Deserialize, Serialize,
};
use sled::IVec;
use thiserror::Error;

pub use sled_rkyv_macros::Collection;
pub mod private {
    pub use lazy_static::lazy_static;
    use parking_lot::{const_mutex, Mutex};
    pub use sled;

    pub static CONFIG: Mutex<Option<sled::Config>> = const_mutex(None);
    lazy_static! {
        pub static ref DB: sled::Db = CONFIG.lock().clone().unwrap_or_default().open().unwrap();
    }
}

pub use sled::Config;
pub fn set_config(config: Config) {
    *private::CONFIG.lock() = Some(config);
}

pub const SERIALIZER_SCRATCH_SPACE: usize = 1024;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    SledRkyvError(String),
    #[error("Sled error: {0}")]
    SledError(#[from] sled::Error),
    #[error("Rkyv serialization error: {0}")]
    RkyvSerError(
        #[from] CompositeSerializerError<Infallible, AllocScratchError, SharedSerializeMapError>,
    ),
    #[error("Rkyv deserialization error: {0}")]
    RkyvDeserError(#[from] SharedDeserializeMapError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq)]
pub struct TypedIVec<T>(IVec, PhantomData<T>);

impl<T: Archive> TypedIVec<T>
where
    T::Archived: Deserialize<T, SharedDeserializeMap>,
{
    pub fn new(raw: IVec) -> Self {
        Self(raw, PhantomData)
    }

    pub fn to_archive(&self) -> Result<T> {
        Ok(unsafe { from_bytes_unchecked(&self.0)? })
    }
}

impl<T: Archive> Deref for TypedIVec<T>
where
    T::Archived: Deserialize<T, SharedDeserializeMap>,
{
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        unsafe { archived_root::<T>(&self.0) }
    }
}

impl<T: Archive + Debug> Debug for TypedIVec<T>
where
    T::Archived: Deserialize<T, SharedDeserializeMap>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedIVec")
            .field("raw", &self.0)
            .field("archived", &self.to_archive())
            .finish()
    }
}

pub trait Collection:
    Archive + Sized + Serialize<AllocSerializer<SERIALIZER_SCRATCH_SPACE>>
where
    Self::Archived: Deserialize<Self, SharedDeserializeMap>,
{
    type Id: ?Sized;

    fn get_id(&self) -> Cow<[u8]>;

    fn build_id(id: &Self::Id) -> Cow<[u8]>;

    fn get_tree() -> &'static sled::Tree;

    fn get(id: &Self::Id) -> Result<Option<TypedIVec<Self>>> {
        Ok(Self::get_tree()
            .get(Self::build_id(id))?
            .map(TypedIVec::new))
    }

    fn insert(&self) -> Result<Option<TypedIVec<Self>>> {
        dbg!(to_bytes(self)?);
        Ok(Self::get_tree()
            .insert(self.get_id(), to_bytes(self)?.as_ref())?
            .map(TypedIVec::new))
    }

    fn remove(id: &Self::Id) -> Result<Option<TypedIVec<Self>>> {
        Ok(Self::get_tree()
            .remove(Self::build_id(id))?
            .map(TypedIVec::new))
    }
}
