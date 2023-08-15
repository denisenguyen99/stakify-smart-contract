#![cfg(test)]
mod tests {
    const MOCK_1000_TOKEN_AMOUNT: u128 = 1_000_000;

    // create a lp token contract
    // create pool contract by factory contract
    // deposit some lp token to the pool contract
    // withdraw some lp token from the pool contract
    mod execute_proper_operation {

        use campaign::msg::{ExecuteMsg as CampaignExecuteMsg, QueryMsg as CampaignQueryMsg};
        use campaign::state::{
            AssetTokenInfo, CampaignInfoResult, LockupTerm, NftInfo, NftStake, StakedInfoResult,
            TokenInfo,
        };
        use cosmwasm_std::{Addr, BlockInfo, Empty, Uint128};
        use cw20::{BalanceResponse, Cw20ExecuteMsg};
        use cw721_base::MintMsg as Cw721MintMsg;
        use cw_multi_test::Executor;

        use crate::{
            msg::QueryMsg,
            state::{FactoryCampaign, Metadata},
            tests::{
                env_setup::env::{instantiate_contracts, ADMIN},
                integration_test::tests::MOCK_1000_TOKEN_AMOUNT,
            },
        };

        pub type Extension = Option<Metadata>;
        pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;

        #[test]
        fn proper_operation() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();

            // get factory contract
            let factory_contract = &contracts[0].contract_addr;
            // get lp token contract
            let token_contract = &contracts[1].contract_addr;
            // get collection contract
            let collection_contract = &contracts[2].contract_addr;

            // Mint 1000 tokens to ADMIN
            let mint_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: Uint128::from(MOCK_1000_TOKEN_AMOUNT),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(token_contract.clone()),
                &mint_msg,
                &[],
            );

            assert!(response.is_ok());

            // mint 5 nft with token_id = 1..5
            for id in 1..5 {
                // mint nft
                let mint_nft_msg = Cw721MintMsg {
                    token_id: id.to_string(),
                    owner: ADMIN.to_string(),
                    token_uri: Some(
                        "https://starships.example.com/Starship/Enterprise.json".into(),
                    ),
                    extension: Some(Metadata {
                        description: Some("Spaceship with Warp Drive".into()),
                        name: Some("Starship USS Enterprise".to_string()),
                        ..Metadata::default()
                    }),
                };

                let exec_msg = Cw721ExecuteMsg::Mint(mint_nft_msg.clone());

                let response_mint_nft = app.execute_contract(
                    Addr::unchecked(ADMIN.to_string()),
                    Addr::unchecked(collection_contract.clone()),
                    &exec_msg,
                    &[],
                );

                assert!(response_mint_nft.is_ok());
            }

            // query balance of ADMIN in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();
            // It should be 1000 lp token as minting happened
            assert_eq!(balance.balance, Uint128::from(MOCK_1000_TOKEN_AMOUNT));

            // token info
            let token_info = TokenInfo::Token {
                contract_addr: token_contract.to_string(),
            };

            // get current block time
            let current_block_time = app.block_info().time.seconds();

            // create campaign contract by factory contract
            let create_campaign_msg = crate::msg::ExecuteMsg::CreateCampaign {
                owner: ADMIN.to_string(),
                campaign_name: "campaign name".to_string(),
                campaign_image: "campaign name".to_string(),
                campaign_description: "campaign name".to_string(),
                start_time: current_block_time + 10,
                end_time: current_block_time + 110,
                limit_per_staker: 5,
                reward_token_info: AssetTokenInfo {
                    info: token_info.clone(),
                    amount: Uint128::zero(),
                },
                allowed_collection: collection_contract.clone(),
                lockup_term: vec![
                    LockupTerm {
                        value: 10,
                        percent: Uint128::new(30u128),
                    },
                    LockupTerm {
                        value: 30,
                        percent: Uint128::new(70u128),
                    },
                ],
            };

            // Execute create campaign
            let response_create_campaign = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &create_campaign_msg,
                &[],
            );

            assert!(response_create_campaign.is_ok());

            // query campaign contract address
            let campaign_info: FactoryCampaign = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &crate::msg::QueryMsg::Campaign { campaign_id: 1u64 },
                )
                .unwrap();

            // assert campaign info
            assert_eq!(
                campaign_info,
                FactoryCampaign {
                    owner: Addr::unchecked(ADMIN.to_string()),
                    campaign_addr: Addr::unchecked("contract3"),
                    reward_token: TokenInfo::Token {
                        contract_addr: token_contract.to_string()
                    },
                    allowed_collection: Addr::unchecked(collection_contract)
                }
            );

            // query campaign contract address
            let campaign_info: CampaignInfoResult = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked("contract3"),
                    &CampaignQueryMsg::CampaignInfo {},
                )
                .unwrap();

            // assert campaign info
            assert_eq!(
                campaign_info,
                CampaignInfoResult {
                    owner: Addr::unchecked(ADMIN.to_string()),
                    campaign_name: "campaign name".to_string(),
                    campaign_image: "campaign name".to_string(),
                    campaign_description: "campaign name".to_string(),
                    limit_per_staker: 5,
                    reward_token_info: AssetTokenInfo {
                        info: token_info.clone(),
                        amount: Uint128::zero(),
                    },
                    allowed_collection: Addr::unchecked(collection_contract.clone()),
                    lockup_term: vec![
                        LockupTerm {
                            value: 10,
                            percent: Uint128::new(30u128),
                        },
                        LockupTerm {
                            value: 30,
                            percent: Uint128::new(70u128),
                        },
                    ],
                    total_nft_staked: 0,
                    total_reward_claimed: Uint128::zero(),
                    total_reward: Uint128::zero(),
                    reward_per_second: Uint128::zero(),
                    start_time: current_block_time + 10,
                    end_time: current_block_time + 110,
                }
            );

            // // update campaign
            // let update_campaign_msg = CampaignExecuteMsg::UpdateCampaign {
            //     campaign_name: "campaign name".to_string(),
            //     campaign_image: "campaign image".to_string(),
            //     campaign_description: "campaign description".to_string(),
            //     limit_per_staker: 5,
            //     lockup_term: vec![
            //         LockupTerm {
            //             value: 10,
            //             percent: Uint128::new(30u128),
            //         },
            //         LockupTerm {
            //             value: 30,
            //             percent: Uint128::new(70u128),
            //         },
            //     ],
            //     start_time: current_block_time + 10,
            //     end_time: current_block_time + 110,
            // };

            // // Execute update campaign
            // let response = app.execute_contract(
            //     Addr::unchecked(ADMIN.to_string()),
            //     Addr::unchecked("contract3"),
            //     &update_campaign_msg,
            //     &[],
            // );

            // assert!(response.is_ok());
            // query all campaigns
            let campaigns: Vec<FactoryCampaign> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(factory_contract.clone()),
                    &QueryMsg::Campaigns {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            // assert campaign info

            // TODO: contract3 unknown ?
            assert_eq!(
                campaigns,
                vec![FactoryCampaign {
                    owner: Addr::unchecked(ADMIN.to_string()),
                    campaign_addr: Addr::unchecked("contract3"),
                    reward_token: TokenInfo::Token {
                        contract_addr: token_contract.to_string()
                    },
                    allowed_collection: Addr::unchecked(collection_contract)
                }]
            );

            // Approve cw20 token to campaign contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract3".to_string(), // Campaign Contract
                amount: Uint128::from(MOCK_1000_TOKEN_AMOUNT),
                expires: None,
            };

            // Execute approve
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(token_contract.clone()),
                &approve_msg,
                &[],
            );

            assert!(response.is_ok());

            // add reward token
            let add_reward_balance_msg = CampaignExecuteMsg::AddRewardToken {
                amount: Uint128::from(MOCK_1000_TOKEN_AMOUNT),
            };

            // Execute add reward balance
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract3"),
                &add_reward_balance_msg,
                &[],
            );

            assert!(response.is_ok());

            // check reward token in campaign
            let campaign_info: CampaignInfoResult = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::CampaignInfo {})
                .unwrap();

            assert_eq!(
                Uint128::from(MOCK_1000_TOKEN_AMOUNT),
                campaign_info.reward_token_info.amount
            );

            // query balance of ADMIN in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // It should be 0 token as deposit happened
            assert_eq!(balance.balance, Uint128::zero());

            // query balance of campaign contract in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: "contract3".to_string(),
                    },
                )
                .unwrap();

            // It should be MOCK_1000_TOKEN_AMOUNT token as deposit happened
            assert_eq!(balance.balance, Uint128::from(MOCK_1000_TOKEN_AMOUNT));

            // increase 20 second to make active campaign
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(20),
                height: app.block_info().height + 20,
                chain_id: app.block_info().chain_id,
            });
            // assert_eq!(app.block_info().time.seconds(), 1);
            // Approve all nft
            for id in 1..5 {
                // Approve nft to campaign contract
                let approve_msg: Cw721ExecuteMsg = Cw721ExecuteMsg::Approve {
                    spender: "contract3".to_string(), // Campaign Contract
                    token_id: id.to_string(),
                    expires: None,
                };

                // Execute approve nft
                let response = app.execute_contract(
                    Addr::unchecked(ADMIN.to_string()),
                    Addr::unchecked(collection_contract.clone()),
                    &approve_msg,
                    &[],
                );
                assert!(response.is_ok());
            }

            // stake nft token_id 1
            let stake_nft_msg = CampaignExecuteMsg::StakeNfts {
                nfts: vec![NftStake {
                    token_id: "1".to_string(),
                    lockup_term: 10,
                }],
            };
            let start_time_1 = app.block_info().time.seconds();

            // Execute stake nft to campaign
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract3"),
                &stake_nft_msg,
                &[],
            );

            assert!(response.is_ok());

            // get nft info
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 1 })
                .unwrap();

            assert_eq!(
                nft_info,
                NftInfo {
                    key: 1,
                    token_id: "1".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(0u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_1,
                    end_time: start_time_1 + 10
                }
            );

            // nft staked with ADMIN
            let staked: StakedInfoResult = app
                .wrap()
                .query_wasm_smart(
                    "contract3",
                    &CampaignQueryMsg::NftStaked {
                        owner: Addr::unchecked(ADMIN.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(
                staked,
                StakedInfoResult {
                    nfts: vec![NftInfo {
                        key: 1,
                        token_id: "1".to_string(),
                        owner_nft: Addr::unchecked(ADMIN.to_string()),
                        pending_reward: Uint128::from(0u128),
                        lockup_term: LockupTerm {
                            value: 10,
                            percent: Uint128::from(30u128)
                        },
                        is_end_reward: false,
                        is_unstake: false,
                        start_time: start_time_1,
                        end_time: start_time_1 + 10
                    }],
                    reward_debt: Uint128::zero(),
                    reward_claimed: Uint128::zero()
                }
            );

            // change block time increase 1 second to next stake nft 2
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(1),
                height: app.block_info().height + 1,
                chain_id: app.block_info().chain_id,
            });

            // stake nft token_id 2
            let stake_nft_msg = CampaignExecuteMsg::StakeNfts {
                nfts: vec![NftStake {
                    token_id: "2".to_string(),
                    lockup_term: 10,
                }],
            };
            let start_time_2 = app.block_info().time.seconds();

            // Execute stake nft to campaign
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract3"),
                &stake_nft_msg,
                &[],
            );

            assert!(response.is_ok());

            // get nft info token_id 1
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 1 })
                .unwrap();

            assert_eq!(
                nft_info,
                NftInfo {
                    key: 1,
                    token_id: "1".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(3000u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_1,
                    end_time: start_time_1 + 10
                }
            );

            // get nft info token_id 2
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 2 })
                .unwrap();

            assert_eq!(
                nft_info,
                NftInfo {
                    key: 2,
                    token_id: "2".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(0u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_2,
                    end_time: start_time_2 + 10
                }
            );

            // stake nft token_id 3
            let stake_nft_msg = CampaignExecuteMsg::StakeNfts {
                nfts: vec![NftStake {
                    token_id: "3".to_string(),
                    lockup_term: 10,
                }],
            };
            let start_time_3 = app.block_info().time.seconds();
            // Execute stake nft to campaign
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract3"),
                &stake_nft_msg,
                &[],
            );

            assert!(response.is_ok());

            // get nft info token_id 1
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 1 })
                .unwrap();

            assert_eq!(
                nft_info,
                NftInfo {
                    key: 1,
                    token_id: "1".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(3000u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_1,
                    end_time: start_time_1 + 10
                }
            );

            // get nft info token_id 2
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 2 })
                .unwrap();

            assert_eq!(
                nft_info,
                NftInfo {
                    key: 2,
                    token_id: "2".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(0u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_2,
                    end_time: start_time_2 + 10
                }
            );

            // get nft info token_id 3
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 3 })
                .unwrap();

            assert_eq!(
                nft_info,
                NftInfo {
                    key: 3,
                    token_id: "3".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(0u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_3,
                    end_time: start_time_3 + 10
                }
            );

            // increase 9 second to make active campaign
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(9),
                height: app.block_info().height + 9,
                chain_id: app.block_info().chain_id,
            });

            // stake nft token_id 4
            let stake_nft_msg = CampaignExecuteMsg::StakeNfts {
                nfts: vec![NftStake {
                    token_id: "4".to_string(),
                    lockup_term: 30,
                }],
            };

            let start_time_4 = app.block_info().time.seconds();

            // Execute stake nft to campaign
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract3"),
                &stake_nft_msg,
                &[],
            );

            assert!(response.is_ok());

            // get nft info token_id 1
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 1 })
                .unwrap();

            // assert_eq!(app.block_info().time.seconds(), 1);
            assert_eq!(
                nft_info,
                NftInfo {
                    key: 1,
                    token_id: "1".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(12000u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: true,
                    is_unstake: false,
                    start_time: start_time_1,
                    end_time: start_time_1 + 10
                }
            );

            // get nft info token_id 2
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 2 })
                .unwrap();

            // assert_eq!(app.block_info().time.seconds(), 1);
            assert_eq!(
                nft_info,
                NftInfo {
                    key: 2,
                    token_id: "2".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(9000u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_2,
                    end_time: start_time_2 + 10
                }
            );

            // get nft info token_id 3
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 3 })
                .unwrap();

            // assert_eq!(app.block_info().time.seconds(), 1);
            assert_eq!(
                nft_info,
                NftInfo {
                    key: 3,
                    token_id: "3".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(9000u128),
                    lockup_term: LockupTerm {
                        value: 10,
                        percent: Uint128::from(30u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_3,
                    end_time: start_time_3 + 10
                }
            );

            // get nft info token_id 4
            let nft_info: NftInfo = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::NftInfo { nft_id: 4 })
                .unwrap();

            // assert_eq!(app.block_info().time.seconds(), 1);
            assert_eq!(
                nft_info,
                NftInfo {
                    key: 4,
                    token_id: "4".to_string(),
                    owner_nft: Addr::unchecked(ADMIN.to_string()),
                    pending_reward: Uint128::from(0u128),
                    lockup_term: LockupTerm {
                        value: 30,
                        percent: Uint128::from(70u128)
                    },
                    is_end_reward: false,
                    is_unstake: false,
                    start_time: start_time_4,
                    end_time: start_time_4 + 30
                }
            );

            // get staker total pending reward
            let total_pending_reward: Uint128 = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::TotalPendingReward {})
                .unwrap();

            // token_id 1 = 12000, token_id 2 = 9000, token_id 3 = 9000, token_id 4 = 0
            assert_eq!(total_pending_reward, Uint128::from(30000u128));

            // increase 10 second to query staker info
            app.set_block(BlockInfo {
                time: app.block_info().time.plus_seconds(10),
                height: app.block_info().height + 10,
                chain_id: app.block_info().chain_id,
            });

            // get staker info
            let staker_info: StakedInfoResult = app
                .wrap()
                .query_wasm_smart(
                    "contract3",
                    &CampaignQueryMsg::NftStaked {
                        owner: Addr::unchecked(ADMIN.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(
                staker_info,
                StakedInfoResult {
                    nfts: vec![
                        NftInfo {
                            key: 1,
                            token_id: "1".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(12000u128),
                            lockup_term: LockupTerm {
                                value: 10,
                                percent: Uint128::from(30u128),
                            },
                            is_end_reward: true,
                            is_unstake: false,
                            start_time: start_time_1,
                            end_time: start_time_1 + 10
                        },
                        NftInfo {
                            key: 2,
                            token_id: "2".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(10500u128),
                            lockup_term: LockupTerm {
                                value: 10,
                                percent: Uint128::from(30u128),
                            },
                            is_end_reward: true,
                            is_unstake: false,
                            start_time: start_time_2,
                            end_time: start_time_2 + 10
                        },
                        NftInfo {
                            key: 3,
                            token_id: "3".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(10500u128),
                            lockup_term: LockupTerm {
                                value: 10,
                                percent: Uint128::from(30u128),
                            },
                            is_end_reward: true,
                            is_unstake: false,
                            start_time: start_time_3,
                            end_time: start_time_3 + 10
                        },
                        NftInfo {
                            key: 4,
                            token_id: "4".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(70000u128),
                            lockup_term: LockupTerm {
                                value: 30,
                                percent: Uint128::from(70u128),
                            },
                            is_end_reward: false,
                            is_unstake: false,
                            start_time: start_time_4,
                            end_time: start_time_4 + 30
                        }
                    ],
                    reward_debt: Uint128::from(103000u128),
                    reward_claimed: Uint128::zero()
                }
            );

            // get staker total pending reward
            let total_pending_reward: Uint128 = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::TotalPendingReward {})
                .unwrap();

            // token_id 1 = 12000, token_id 2 = 10500, token_id 3 = 10500, token_id 4 = 70000
            assert_eq!(total_pending_reward, Uint128::from(103000u128));

            // claim reward msg
            let claim_reward_msg = CampaignExecuteMsg::ClaimReward {
                amount: Uint128::from(100000u128),
            };

            // Execute claim reward
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract3"),
                &claim_reward_msg,
                &[],
            );

            assert!(response.is_ok());

            // get staker info
            let staker_info: StakedInfoResult = app
                .wrap()
                .query_wasm_smart(
                    "contract3",
                    &CampaignQueryMsg::NftStaked {
                        owner: Addr::unchecked(ADMIN.to_string()),
                    },
                )
                .unwrap();

            assert_eq!(
                staker_info,
                StakedInfoResult {
                    nfts: vec![
                        NftInfo {
                            key: 1,
                            token_id: "1".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(0u128),
                            lockup_term: LockupTerm {
                                value: 10,
                                percent: Uint128::from(30u128),
                            },
                            is_end_reward: true,
                            is_unstake: false,
                            start_time: start_time_1,
                            end_time: start_time_1 + 10
                        },
                        NftInfo {
                            key: 2,
                            token_id: "2".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(0u128),
                            lockup_term: LockupTerm {
                                value: 10,
                                percent: Uint128::from(30u128),
                            },
                            is_end_reward: true,
                            is_unstake: false,
                            start_time: start_time_2,
                            end_time: start_time_2 + 10
                        },
                        NftInfo {
                            key: 3,
                            token_id: "3".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(0u128),
                            lockup_term: LockupTerm {
                                value: 10,
                                percent: Uint128::from(30u128),
                            },
                            is_end_reward: true,
                            is_unstake: false,
                            start_time: start_time_3,
                            end_time: start_time_3 + 10
                        },
                        NftInfo {
                            key: 4,
                            token_id: "4".to_string(),
                            owner_nft: Addr::unchecked(ADMIN.to_string()),
                            pending_reward: Uint128::from(0u128),
                            lockup_term: LockupTerm {
                                value: 30,
                                percent: Uint128::from(70u128),
                            },
                            is_end_reward: false,
                            is_unstake: false,
                            start_time: start_time_4,
                            end_time: start_time_4 + 30
                        }
                    ],
                    reward_debt: Uint128::from(3000u128),
                    reward_claimed: Uint128::from(100000u128),
                }
            );

            // get staker total pending reward
            let total_pending_reward: Uint128 = app
                .wrap()
                .query_wasm_smart("contract3", &CampaignQueryMsg::TotalPendingReward {})
                .unwrap();

            // total claimed = 100000
            assert_eq!(total_pending_reward, Uint128::from(3000u128));

        }
    }
}
