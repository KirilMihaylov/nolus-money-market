use serde::{Deserialize, Serialize};

#[cfg(feature = "schema")]
use sdk::schemars::{self, JsonSchema};
use sdk::{
    cosmwasm_std::{StdError, StdResult, Storage},
    cw_storage_plus::Item,
};

mod release;
pub use release::release;

pub type VersionSegment = u16;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SemVer {
    major: VersionSegment,
    minor: VersionSegment,
    patch: VersionSegment,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Version {
    storage: VersionSegment,
    software: SemVer,
}

impl Version {
    pub fn new(storage: VersionSegment, software: SemVer) -> Self {
        Self { storage, software }
    }
}

pub fn parse_semver(version: &str) -> SemVer {
    fn parse_segment<'r, I>(
        iter: &mut I,
        lowercase_name: &str,
        pascal_case_name: &str,
    ) -> VersionSegment
    where
        I: Iterator<Item = &'r str> + ?Sized,
    {
        iter.next()
            .unwrap_or_else(|| panic!("No {} segment in version string!", lowercase_name))
            .parse()
            .unwrap_or_else(|_| {
                panic!(
                    "{} segment in version string is not a number!",
                    pascal_case_name
                )
            })
    }

    let mut iter = version.split('.');

    let major: VersionSegment = parse_segment(&mut iter, "major", "Major");
    let minor: VersionSegment = parse_segment(&mut iter, "minor", "Minor");
    let patch: VersionSegment = parse_segment(&mut iter, "patch", "Patch");

    if iter.next().is_some() {
        panic!("Unexpected fourth segment found in version string!");
    };

    SemVer {
        major,
        minor,
        patch,
    }
}

#[macro_export]
macro_rules! package_version {
    () => {{
        $crate::parse_semver(::core::env!(
            "CARGO_PKG_VERSION",
            "Cargo package version is not set as an environment variable!",
        ))
    }};
}

#[macro_export]
macro_rules! version {
    ($storage: expr) => {{
        $crate::Version::new($storage, $crate::package_version!())
    }};
}

const VERSION_STORAGE_KEY: Item<'static, Version> = Item::new("contract_version");

pub fn initialize(storage: &mut dyn Storage, version: Version) -> StdResult<()> {
    VERSION_STORAGE_KEY.save(storage, &version)
}

#[inline]
pub fn update_software<ContractError>(
    storage: &mut dyn Storage,
    new_version: Version,
) -> Result<(), ContractError>
where
    StdError: Into<ContractError>,
{
    update_version(storage, new_version.storage, new_version).map(|_| ())
}

pub fn update_software_and_storage<
    const FROM_STORAGE_VERSION: VersionSegment,
    MigrateStorageFunctor,
    ContractError,
>(
    storage: &mut dyn Storage,
    new_version: Version,
    migrate_storage: MigrateStorageFunctor,
) -> Result<(), ContractError>
where
    MigrateStorageFunctor: FnOnce(&mut dyn Storage) -> Result<(), ContractError>,
    StdError: Into<ContractError>,
{
    if new_version.storage == FROM_STORAGE_VERSION {
        return Err(StdError::generic_err("Software and storage update handler called, but expected and new storage versions are the same!").into());
    }

    if new_version.storage != FROM_STORAGE_VERSION.wrapping_add(1) {
        return Err(StdError::generic_err("Expected and new storage versions are not directly adjacent! This could indicate an error!").into());
    }

    update_version(storage, FROM_STORAGE_VERSION, new_version).map_err(Into::into)?;

    migrate_storage(storage)
}

// trait Release {
//     fn allow_update(&self, current: &Version, new: &Version) -> Result<(), MigrateStorageError>;
// }

fn update_version<ContractError>(
    storage: &mut dyn Storage,
    expected_storage: VersionSegment,
    new_version: Version,
) -> Result<Version, ContractError>
where
    StdError: Into<ContractError>,
{
    VERSION_STORAGE_KEY.update(storage, |saved_version| {
        if saved_version.storage != expected_storage {
            return Err(StdError::generic_err(format!(
                "Software update handler called, but storage versions differ! Saved storage version is {saved}, but storage version used by this software is {current}!",
                saved = saved_version.storage,
                current = expected_storage,
            )));
        }

        if saved_version.software < new_version.software
            || (release::dev_release() && saved_version.software == new_version.software) {
            Ok(new_version)
        } else {
            Err(StdError::generic_err(
                "Couldn't upgrade contract because software version isn't monotonically increasing!",
            ))
        }
    }).map_err(Into::into)
}
