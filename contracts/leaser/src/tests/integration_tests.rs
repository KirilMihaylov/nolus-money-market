#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, Coin, Empty, Uint64};
    use cw_multi_test::{next_block, App, AppBuilder, Contract, ContractWrapper, Executor};
    use finance::{liability::Liability, percent::Percent};
    use lease::msg::{LoanForm, NewLeaseForm};
    use lpp::msg::InstantiateMsg as LppInstantiateMsg;

    use crate::{
        msg::{QueryMsg, QuoteResponse},
        tests::common::leaser_instantiate_msg,
    };

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";

    pub fn contract_leaser_mock() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply);
        Box::new(contract)
    }

    pub fn contract_lease_mock() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            lease::contract::execute,
            lease::contract::instantiate,
            lease::contract::query,
        )
        .with_reply(lease::contract::reply);
        Box::new(contract)
    }

    pub fn contract_lpp_mock() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            lpp::contract::execute,
            lpp::contract::instantiate,
            lpp::contract::query,
        );
        Box::new(contract)
    }

    fn mock_app(init_funds: &[Coin]) -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(ADMIN), init_funds.to_vec())
                .unwrap();
        })
    }

    #[track_caller]
    fn instantiate_leaser(app: &mut App, lease_code_id: u64, lpp_addr: Addr) -> Addr {
        let leaser_id = app.store_code(contract_leaser_mock());
        let msg = leaser_instantiate_msg(lease_code_id, lpp_addr);
        app.instantiate_contract(leaser_id, Addr::unchecked(ADMIN), &msg, &[], "leaser", None)
            .unwrap()
    }

    #[track_caller]
    fn _instantiate_lease(app: &mut App, lease_id: u64, lpp_addr: Addr, denom: &str) -> Addr {
        let msg = NewLeaseForm {
            customer: USER.to_string(),
            currency: denom.to_string(),
            liability: Liability::new(
                Percent::from(65),
                Percent::from(5),
                Percent::from(10),
                20 * 24,
            ),
            loan: LoanForm {
                annual_margin_interest_permille: 0, // 3.1%
                lpp: lpp_addr.into_string(),
                interest_due_period_secs: 100, // 90 days TODO use a crate for daytime calculations
                grace_period_secs: 10,         // 10 days TODO use a crate for daytime calculations
            },
        };

        app.instantiate_contract(
            lease_id,
            Addr::unchecked(ADMIN),
            &msg,
            &coins(400, denom),
            "lease",
            None,
        )
        .unwrap()
    }

    #[track_caller]
    fn instantiate_lpp(app: &mut App, lease_code_id: Uint64, denom: &str) -> (Addr, u64) {
        let lpp_id = app.store_code(contract_lpp_mock());
        let msg = LppInstantiateMsg {
            denom: denom.to_string(),
            lease_code_id,
        };
        (
            app.instantiate_contract(
                lpp_id,
                Addr::unchecked(ADMIN),
                &msg,
                &coins(400, denom),
                "lpp",
                None,
            )
            .unwrap(),
            lpp_id,
        )
    }

    pub fn setup_test_case(
        app: &mut App,
        init_funds: Vec<Coin>,
        user_addr: Addr,
        denom: &str,
    ) -> (Addr, u64) {
        let lease_id = app.store_code(contract_lease_mock());

        // 1. Instantiate LPP contract
        let (lpp_addr, _lpp_id) = instantiate_lpp(app, Uint64::new(lease_id), denom);
        app.update_block(next_block);

        // // // 2. Instantiate Lease contract (and OWNER as admin)
        // let _lease_addr = instantiate_lease(app, lease_id, lpp_addr.clone());
        // app.update_block(next_block);

        // 3. Instantiate Leaser contract
        let leaser_addr = instantiate_leaser(app, lease_id, lpp_addr);
        app.update_block(next_block);

        // Bonus: set some funds on the user for future proposals
        if !init_funds.is_empty() {
            app.send_tokens(Addr::unchecked(ADMIN), user_addr, &init_funds)
                .unwrap();
        }
        (leaser_addr, lease_id)
    }

    #[test]
    fn open_lease() {
        let denom = "UST";
        let mut app = mock_app(&coins(10000, denom));
        let user_addr = Addr::unchecked(USER);

        let (leaser_addr, lease_code_id) =
            setup_test_case(&mut app, coins(500, denom), user_addr.clone(), denom);

        assert_eq!(
            coins(500, denom),
            app.wrap().query_all_balances(user_addr.clone()).unwrap()
        );

        let res = app
            .execute_contract(
                user_addr.clone(),
                leaser_addr.clone(),
                &crate::msg::ExecuteMsg::OpenLease {
                    currency: denom.to_string(),
                },
                &coins(40, denom),
            )
            .unwrap();

        // ensure the attributes were relayed from the sub-message
        assert_eq!(8, res.events.len(), "{:?}", res.events);
        // reflect only returns standard wasm-execute event
        let leaser_exec = &res.events[0];
        assert_eq!(leaser_exec.ty.as_str(), "execute");
        assert_eq!(leaser_exec.attributes, [("_contract_addr", &leaser_addr)]);

        let lease_exec = &res.events[1];
        assert_eq!(lease_exec.ty.as_str(), "instantiate");
        assert_eq!(
            lease_exec.attributes,
            [
                ("_contract_addr", "Contract #2"),
                ("code_id", &lease_code_id.to_string())
            ]
        );

        let lease_reply = &res.events[2];
        assert_eq!(lease_reply.ty.as_str(), "execute");
        assert_eq!(lease_reply.attributes, [("_contract_addr", "Contract #0")]);

        let lease_reply = &res.events[3];
        assert_eq!(lease_reply.ty.as_str(), "wasm");
        assert_eq!(
            lease_reply.attributes,
            [
                ("_contract_addr", "Contract #0"),
                ("method", "try_open_loan")
            ]
        );

        let lease_reply = &res.events[4];
        assert_eq!(lease_reply.ty.as_str(), "transfer");
        assert_eq!(
            lease_reply.attributes,
            [
                ("recipient", "Contract #2"),
                ("sender", "Contract #0"),
                ("amount", "74UST")
            ]
        );

        let lease_reply = &res.events[5];
        assert_eq!(lease_reply.ty.as_str(), "reply");
        assert_eq!(
            lease_reply.attributes,
            [
                ("_contract_addr", "Contract #2"),
                ("mode", "handle_success")
            ]
        );

        let lease_reply = &res.events[6];
        assert_eq!(lease_reply.ty.as_str(), "reply");
        assert_eq!(
            lease_reply.attributes,
            [
                ("_contract_addr", "Contract #1"),
                ("mode", "handle_success")
            ]
        );

        let lease_reply = &res.events[7];
        assert_eq!(lease_reply.ty.as_str(), "wasm");
        assert_eq!(
            lease_reply.attributes,
            [
                ("_contract_addr", leaser_addr.as_str()),
                ("lease_address", "Contract #2")
            ]
        );

        let lease_address = &res.events[7].attributes.get(1).unwrap().value;

        assert_eq!(
            coins(460, denom),
            app.wrap().query_all_balances(user_addr).unwrap()
        );
        assert_eq!(
            coins(114, denom),
            app.wrap().query_all_balances(lease_address).unwrap()
        );
    }

    #[test]
    fn open_lease_custom_currency() {
        let denom = "unolus";
        let mut app = mock_app(&coins(10000, denom));
        let user_addr = Addr::unchecked(USER);

        let (leaser_addr, _) =
            setup_test_case(&mut app, coins(500, denom), user_addr.clone(), denom);

        assert_eq!(
            coins(500, denom),
            app.wrap().query_all_balances(user_addr.clone()).unwrap()
        );

        let res = app
            .execute_contract(
                user_addr.clone(),
                leaser_addr,
                &crate::msg::ExecuteMsg::OpenLease {
                    currency: denom.to_string(),
                },
                &coins(3, denom),
            )
            .unwrap();

        let lease_address = &res.events[7].attributes.get(1).unwrap().value;

        assert_eq!(
            coins(497, denom),
            app.wrap().query_all_balances(user_addr).unwrap()
        );
        assert_eq!(
            coins(5, denom),
            app.wrap().query_all_balances(lease_address).unwrap()
        );
    }

    #[test]
    fn test_quote() {
        let denom = "UST";
        let mut app = mock_app(&coins(10000, denom));
        let user_addr = Addr::unchecked(USER);
        let (leaser_addr, _) = setup_test_case(&mut app, coins(500, denom), user_addr, denom);

        let resp: QuoteResponse = app
            .wrap()
            .query_wasm_smart(
                leaser_addr,
                &QueryMsg::Quote {
                    downpayment: Coin::new(100, denom),
                },
            )
            .unwrap();

        assert_eq!(185, resp.borrow.amount.u128());
    }
}
