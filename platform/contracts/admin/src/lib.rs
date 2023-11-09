use access_control::ContractOwnerAccess;
use platform::{batch::Batch, response};
#[cfg(feature = "contract-with-bindings")]
use sdk::cosmwasm_std::entry_point;
use sdk::{
    cosmwasm_ext::Response as CwResponse,
    cosmwasm_std::{
        ensure_eq, Addr, Binary, DepsMut, Env, MessageInfo, QuerierWrapper, Reply, Storage, WasmMsg,
    },
};
use versioning::{package_version, version, ReleaseLabel, SemVer, Version, VersionSegment};

use self::{
    common::{type_defs::ContractsGroupedByDex, CheckedAddr, Protocol, Transform as _},
    error::Error as ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, SudoMsg},
    result::Result as ContractResult,
    state::{contracts as state_contracts, migration_release},
};

pub mod common;
pub mod error;
pub mod migrate_contracts;
pub mod msg;
pub mod result;
pub mod state;

// version info for migration info
const CONTRACT_STORAGE_VERSION_FROM: VersionSegment = 0;
const CONTRACT_STORAGE_VERSION: VersionSegment = 1;
const PACKAGE_VERSION: SemVer = package_version!();
const CONTRACT_VERSION: Version = version!(CONTRACT_STORAGE_VERSION, PACKAGE_VERSION);

#[cfg_attr(feature = "contract-with-bindings", entry_point)]
pub fn instantiate(
    mut deps: DepsMut<'_>,
    _env: Env,
    _info: MessageInfo,
    InstantiateMsg::Instantiate {
        contract_owner,
        contracts,
    }: InstantiateMsg,
) -> ContractResult<CwResponse> {
    versioning::initialize(deps.storage, CONTRACT_VERSION)?;

    ContractOwnerAccess::new(deps.branch().storage).grant_to(&contract_owner)?;

    contracts
        .transform(&deps.querier)
        .and_then(|contracts: ContractsGroupedByDex| {
            state_contracts::store(deps.storage, contracts).map(|()| response::empty_response())
        })
}

#[cfg_attr(feature = "contract-with-bindings", entry_point)]
pub fn migrate(
    mut deps: DepsMut<'_>,
    _env: Env,
    MigrateMsg::Migrate {
        dex,
        contract_owner,
    }: MigrateMsg,
) -> ContractResult<CwResponse> {
    ContractOwnerAccess::new(deps.branch().storage)
        .grant_to(&contract_owner)
        .map_err(Into::into)
        .and_then(|()| {
            versioning::update_software_and_storage::<CONTRACT_STORAGE_VERSION_FROM, _, _, _, _>(
                deps.storage,
                CONTRACT_VERSION,
                |storage: &mut dyn Storage| state_contracts::migrate(storage, dex),
                Into::into,
            )
        })
        .and_then(|(label, ()): (ReleaseLabel, ())| response::response(label))
}

#[cfg_attr(feature = "contract-with-bindings", entry_point)]
pub fn execute(
    mut deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<CwResponse> {
    ContractOwnerAccess::new(deps.branch().storage).check(&info.sender)?;

    match msg {
        ExecuteMsg::Instantiate {
            code_id,
            label,
            message,
        } => {
            let mut batch: Batch = Batch::default();

            batch.schedule_execute_no_reply(WasmMsg::Instantiate {
                admin: Some(env.contract.address.into_string()),
                code_id,
                msg: Binary(message.into_bytes()),
                funds: info.funds,
                label,
            });

            Ok(response::response_only_messages(batch))
        }
        ExecuteMsg::AddProtocolSet { dex, contracts } => {
            add_protocol_set(deps.storage, &deps.querier, dex, contracts)
        }
    }
}

#[cfg_attr(feature = "contract-with-bindings", entry_point)]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> ContractResult<CwResponse> {
    match msg {
        SudoMsg::ChangeOwner { address } => ContractOwnerAccess::new(deps.storage)
            .grant_to(&address)
            .map(|()| response::empty_response())
            .map_err(Into::into),
        SudoMsg::AddProtocolSet { dex, contracts } => {
            add_protocol_set(deps.storage, &deps.querier, dex, contracts)
        }
        SudoMsg::MigrateContracts(migrate_contracts) => env
            .contract
            .address
            .transform(&deps.querier)
            .and_then(|admin_contract_addr: CheckedAddr| {
                migrate_contracts::migrate(deps.storage, admin_contract_addr, migrate_contracts)
            })
            .map(response::response_only_messages),
    }
}

fn add_protocol_set(
    storage: &mut dyn Storage,
    querier: &QuerierWrapper<'_>,
    dex: String,
    contracts: Protocol<Addr>,
) -> ContractResult<CwResponse> {
    contracts
        .transform(querier)
        .and_then(|contracts: Protocol<CheckedAddr>| {
            state_contracts::add_dex_bound_set(storage, dex, contracts)
        })
        .map(|()| response::empty_response())
}

#[cfg_attr(feature = "contract-with-bindings", entry_point)]
pub fn reply(deps: DepsMut<'_>, _env: Env, msg: Reply) -> ContractResult<CwResponse> {
    let expected_release: String = migration_release::load(deps.storage)?;

    let reported_release: String =
        platform::reply::from_execute(msg)?.ok_or(ContractError::NoMigrationResponseData {})?;

    ensure_eq!(
        reported_release,
        expected_release,
        ContractError::WrongRelease {
            reported: reported_release,
            expected: expected_release
        }
    );

    Ok(response::empty_response())
}
