mod crates_io;
mod npm;
mod pypi;
mod rubygems;

pub(crate) use crates_io::StaticCrateClient;
pub(crate) use npm::StaticPackumentClient;
pub(crate) use pypi::StaticProjectClient;
pub(crate) use rubygems::StaticVersionsClient;
