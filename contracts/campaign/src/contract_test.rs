#[cfg(test)]
mod unit_tests {
    use crate::ContractError;
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{ExecuteMsg, InstantiateMsg};
    use crate::state::{AssetTokenInfo, LockupTerm, TokenInfo, CAMPAIGN_INFO};

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        coins, to_binary, Addr, BankMsg, BlockInfo, Coin, ContractInfo, CosmosMsg, Env, HexBinary,
        OwnedDeps, ReplyOn, Response, SubMsg, SubMsgResult, Timestamp, Uint128, WasmMsg,
    };


    fn get_init_msg(
        owner: &str,
        campaign_name: &str,
        campaign_description: &str,
        start_time: u64,
        end_time: u64,
    ) -> InstantiateMsg {
        InstantiateMsg {
            owner: owner.to_string(),
            campaign_name: campaign_name.to_string(),
            campaign_image: "".to_string(),
            campaign_description: campaign_description.to_string(),
            limit_per_staker: 0,
            reward_token_info: AssetTokenInfo {
                info: TokenInfo::Token {
                    contract_addr: "token_contract_address".to_string(),
                },
                amount: Uint128::zero(),
            },
            allowed_collection: "nft_collection_contract_address".to_string(),
            lockup_term: vec![LockupTerm {
                value: 0,
                percent: Uint128::zero(),
            }],
            start_time,
            end_time,
        }
    }

    const CREATOR: &str = "creator";
    const USER: &str = "user";
    const NOIS_PROXY: &str = "nois proxy";

    // SETUP ENVIROMENT

    fn default_setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let msg = InstantiateMsg {
            owner: CREATOR.to_string(),
            campaign_name: "test".to_string(),
            campaign_image: "".to_string(),
            campaign_description: "campaign_description".to_string(),
            limit_per_staker: 0,
            reward_token_info: AssetTokenInfo {
                info: TokenInfo::Token {
                    contract_addr: "token_contract_address".to_string(),
                },
                amount: Uint128::zero(),
            },
            allowed_collection: "nft_collection_contract_address".to_string(),
            lockup_term: vec![LockupTerm {
                value: 0,
                percent: Uint128::zero(),
            }],
            start_time:env.block.time.seconds(),
            end_time:env.block.time.seconds() + 86400,
        };

        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // assert_eq!(0, res.messages.len());
        assert_eq!(res.attributes.len(), 11);
        return deps;
    }

    // #[test]
    // fn instantiate_works() {
    //     default_setup();
    // }

    #[test]
    fn test_instantiate() {
        // Mock the contract dependencies and environment
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        // Case 1: Valid instantiation with valid parameters
        let init_msg: InstantiateMsg = get_init_msg(
            "creator",
            "Campaign 1",
            "This is a sample campaign",
            env.block.time.seconds(),
            env.block.time.seconds() + 86400, // 1 day after start time
        );

        // Call the instantiate function
        let res=
            instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg);

        // Assert the result
        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(response.attributes.len(), 11); // Check the number of emitted attributes
                                                  // Add more assertions here to check the emitted attributes' values, if needed

        // Case 2: Invalid instantiation with end time greater than max allowed duration (3 years)
        let init_msg_invalid_duration = get_init_msg(
            "creator",
            "Campaign 2",
            "This campaign has an invalid duration",
            env.block.time.seconds(),
            env.block.time.seconds() + 94608001, // 3 years and 1 second after start time
        );

        // Call the instantiate function
        let res_invalid_duration = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            init_msg_invalid_duration,
        );

        // Assert the result (should return an error)
        assert!(res_invalid_duration.is_err());
        match res_invalid_duration.unwrap_err() {
            ContractError::LimitStartDate {} => {}
            _ => panic!("unexpected error"),
        }

        // Case 3: Invalid instantiation with campaign name exceeding character limit
        let init_msg_invalid_name = get_init_msg(
            "creator",
            &"A".repeat(101), // 101 characters long name (exceeding the limit of 100)
            "This campaign has an invalid name",
            env.block.time.seconds(),
            env.block.time.seconds() + 86400,
        );

        // Call the instantiate function
        let res_invalid_name = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            init_msg_invalid_name,
        );

        // Assert the result (should return an error)
        assert!(res_invalid_name.is_err());
        match res_invalid_name.unwrap_err() {
            ContractError::LimitCharacter {} => {}
            _ => panic!("unexpected error"),
        }

        // Case 4: Invalid instantiation with campaign description exceeding character limit
        let init_msg_invalid_description = get_init_msg(
            "creator",
            "Campaign 3",
            &"A".repeat(501), // 501 characters long description (exceeding the limit of 500)
            env.block.time.seconds(),
            env.block.time.seconds() + 86400,
        );

        // Call the instantiate function
        let res_invalid_description = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            init_msg_invalid_description,
        );

        // Assert the result (should return an error)
        assert!(res_invalid_description.is_err());
        match res_invalid_description.unwrap_err() {
            ContractError::LimitCharacter {} => {}
            _ => panic!("unexpected error"),
        }
    }

    #[test]
    fn test_execute_add_reward_token() {
        // Mock the contract dependencies and environment
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        let execute_msg = ExecuteMsg::AddRewardToken { amount: Uint128::new(500) };
        // let res = execute(deps.as_mut(), env.clone(), info.clone(), execute_msg);

        // assert!(res.is_ok());
        // let response = res.unwrap();
        // assert_eq!(response.messages.len(), 4);

        let campaign = CAMPAIGN_INFO.load(&deps.storage);
        // print!(campaign);
        // assert!(campaign.is_ok());
        // let c = campaign.unwrap();
        // assert_eq!(c.reward_token_info.amount, Uint128::new(500));

    }
}
