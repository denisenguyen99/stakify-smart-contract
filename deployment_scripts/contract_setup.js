const chainConfig = require("./config/chain").defaultChain;

const fs = require("fs");

const { SigningCosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet, coin } = require("@cosmjs/proto-signing");
const { calculateFee, GasPrice } = require("@cosmjs/stargate");

// wasm folder
const wasmFolder = `${__dirname}/../artifacts`;

// gas price
const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);
// tester and deployer info
let testerWallet, testerClient, testerAccount;
let deployerWallet, deployerClient, deployerAccount;

/// @dev Store the contract source code on chain
/// @param `wasm_name` - The name of the wasm file
/// @return `storeCodeResponse` - The response of the store code transaction
async function store_contract(wasm_name) {
    const uploadFee = calculateFee(3000000, gasPrice);
    const contractCode = fs.readFileSync(`${wasmFolder}/${wasm_name}.wasm`);

    console.log("Uploading contract code...");
    const storeCodeResponse = await deployerClient.upload(
        deployerAccount.address,
        contractCode,
        uploadFee,
        "Upload campaign contract code"
    );

    console.log("  transactionHash: ", storeCodeResponse.transactionHash);
    console.log("  codeId: ", storeCodeResponse.codeId);
    console.log("  gasWanted / gasUsed: ", storeCodeResponse.gasWanted, " / ", storeCodeResponse.gasUsed);

    return storeCodeResponse;
}

/// @dev Instantiate contract base on the code id and instantiate message of contract
/// @param `_codeID` - The code id of the contract
/// @param `instantiateMsg` - The instantiate message of the contract
/// @return `instantiateResponse` - The response of the instantiate transaction
async function instantiate(contract_code_id, instantiateMsg) {
    console.log("Instantiating contract...");

    //Instantiate the contract
    const instantiateResponse = await deployerClient.instantiate(
        deployerAccount.address,
        Number(contract_code_id),
        instantiateMsg,
        "instantiation contract",
        "auto"
    );
    console.log("  transactionHash: ", instantiateResponse.transactionHash);
    console.log("  contractAddress: ", instantiateResponse.contractAddress);
    console.log("  gasWanted / gasUsed: ", instantiateResponse.gasWanted, " / ", instantiateResponse.gasUsed);

    return instantiateResponse;
}

/// @dev Execute a message to the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `userAccount` -  The account of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `executeMsg` - The message that will be executed
/// @return `executeResponse` - The response of the execute transaction
async function execute(
    userClient,
    userAccount,
    contract,
    executeMsg,
    native_amount = 0,
    native_denom = chainConfig.denom
) {
    console.log("Executing message to contract...");

    const memo = "execute a message";

    let executeResponse;

    // if the native amount is not 0, then send the native token to the contract
    if (native_amount != 0) {
        executeResponse = await userClient.execute(userAccount.address, contract, executeMsg, "auto", memo, [
            coin(native_amount, native_denom),
        ]);
    } else {
        executeResponse = await userClient.execute(userAccount.address, contract, executeMsg, "auto", memo);
    }

    console.log("  transactionHash: ", executeResponse.transactionHash);
    console.log("  gasWanted / gasUsed: ", executeResponse.gasWanted, " / ", executeResponse.gasUsed);

    return executeResponse;
}

/// @dev Query information from the contract
/// @param `userClient` - The client of the user who execute the message
/// @param `contract` - The address of the contract
/// @param `queryMsg` - The message that will be executed
/// @return `queryResponse` - The response of the query
async function query(userClient, contract, queryMsg) {
    console.log("Querying contract...");

    const queryResponse = await userClient.queryContractSmart(contract, queryMsg);

    console.log("  Querying successful");

    return queryResponse;
}

async function main(contract_name) {
    // ***************************
    // SETUP INFORMATION FOR USERS
    // ***************************
    // connect deployer wallet to chain and get deployer account
    deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(chainConfig.deployer_mnemonic, {
        prefix: chainConfig.prefix,
    });
    // console.log({ deployerWallet });
    deployerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, {
        gasPrice,
    });

    // console.log({ deployerClient });
    deployerAccount = (await deployerWallet.getAccounts())[0];
    // console.log({ deployerAccount });

    // connect tester wallet to chain and get tester account
    testerWallet = await DirectSecp256k1HdWallet.fromMnemonic(chainConfig.tester_mnemonic, {
        prefix: chainConfig.prefix,
    });
    testerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, testerWallet, { gasPrice });
    testerAccount = (await testerWallet.getAccounts())[0];

    // ****************
    // EXECUTE CONTRACT
    // ****************
    // store contract
    console.log("1. Storing source code...");
    let storeCodeResponse = await store_contract(contract_name);

    // prepare instantiate message
    // const instantiateMsg = {
    //     owner: "aura148cy455pkrqx8etf4fr57ejy5r9j9yy8cfxlgt",
    //     campaign_name: "Campaign test (Owner: Tri)",
    //     campaign_image:
    //         "https://aura-explorer-assets.s3.ap-southeast-1.amazonaws.com/euphoria-assets/images/icons/aura.svg",
    //     campaign_description: "Campaign test description",
    //     limit_per_staker: 2,
    //     reward_token_info: {
    //         info: {
    //             token: {
    //                 contract_addr: "aura1jv0j23ml9d0mc2v5x5qk9mzcc3zq9suvehppprs5k6cqc37m6tzq6kv33k",
    //             },
    //         },
    //         amount: "0",
    //     },
    //     allowed_collection: "aura163gwaw6gc6xyyrx2p633pykrcs674fy739fjn0gk7h50n5fz8v2qydh65h",
    //     lockup_term: [
    //         {
    //             value: 3600,
    //             percent: "10",
    //         },
    //         {
    //             value: 86400,
    //             percent: "90",
    //         },
    //     ],
    //     start_time: 1690360422,
    //     end_time: 1690615225,
    // };
    const instantiateMsg = {
        campaign_code_id: 1343,
        allow_create_for_all: false,
    };

    // instantiate contract
    console.log("2. Instantiating contract...");
    let instantiateResponse = await instantiate(storeCodeResponse.codeId, instantiateMsg);

    console.log("Contract setup completed!");
}

const myArgs = process.argv.slice(2);
if (myArgs.length != 1) {
    console.log("Usage: node contract_setup.js <wasm_contract_name>");
    process.exit(1);
}
main(myArgs[0]);
