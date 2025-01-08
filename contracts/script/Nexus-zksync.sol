// SPDX-License-Identifier: Apaache 2.0
pragma solidity ^0.8.21;

import "forge-std/console.sol";
import {Script} from "forge-std/Script.sol";
import {NexusMailbox} from "../src/NexusMailbox.sol";
import {NexusProofManager} from "../src/NexusProofManager.sol";
import {ZKSyncNexusManagerRouter} from "../src/verification/zksync/ZKSyncNexusManagerRouter.sol";
import {SparseMerkleTree} from "../src/verification/zksync/SparseMerkleTree.sol";
import {VerifierWrapper} from "../src/verification/zksync/VerifierWrapper.sol";
import {IZKSyncNexusManagerRouter} from "../src/verification/zksync/StorageProof.sol";
import {INexusProofManager} from "../src/interfaces/INexusProofManager.sol";
import {VerifierInfo} from "../src/interfaces/INexusMailbox.sol";
import {INexusVerifierWrapper} from "../src/interfaces/INexusVerifierWrapper.sol";

contract NexusDeployment is Script {
    struct NetworkConfig {
        uint256 deployerPrivateKey;
        bytes32 appId;
        bytes32 appId2;
    }

    NetworkConfig config;

    function setUp() public {
        getConfig();
    }

    function getConfig() internal {
        string memory configFile = vm.envString("CONFIG_FILE");
        string memory network = vm.envString("NETWORK");
        string memory jsonConfig = vm.readFile(configFile);
        string memory basePath = string.concat(".", network);

        // Parse privateKey
        string memory privateKeyPath = string.concat(basePath, ".privateKey");
        config.deployerPrivateKey = abi.decode(
            vm.parseJson(jsonConfig, privateKeyPath),
            (uint256)
        );

        // Parse appId
        string memory appIdPath = string.concat(basePath, ".appId");
        bytes32 appIdUint = abi.decode(
            vm.parseJson(jsonConfig, appIdPath),
            (bytes32)
        );
        config.appId = appIdUint;

        string memory appId2Path = string.concat(basePath, ".appId2");
        bytes32 appId2Uint = abi.decode(
            vm.parseJson(jsonConfig, appId2Path),
            (bytes32)
        );
        config.appId2 = appId2Uint;
    }

    function run() public {
        vm.startBroadcast(config.deployerPrivateKey);

        // Deploy NexusProofManager
        NexusProofManager nexusManager = new NexusProofManager();
        console.log("NexusProofManager deployed to: ", address(nexusManager));

        // Deploy and initialize NexusMailbox
        NexusMailbox mailbox = new NexusMailbox();
        mailbox.initialize(config.appId);
        console.log("Mailbox deployed to: ", address(mailbox));

        // Deploy ZKSyncNexusManagerRouter
        ZKSyncNexusManagerRouter zksyncdiamond = new ZKSyncNexusManagerRouter(
            INexusProofManager(address(nexusManager)),
            config.appId2
        );

        // Deploy SparseMerkleTree
        SparseMerkleTree sparseMerkleTree = new SparseMerkleTree();

        // Deploy VerifierWrapper
        VerifierWrapper verifierWrapper = new VerifierWrapper(
            IZKSyncNexusManagerRouter(address(zksyncdiamond)),
            sparseMerkleTree
        );

        console.log("Verifer deployed to : ", address(verifierWrapper));
        // Add or update wrapper in mailbox
        mailbox.addOrUpdateWrapper(
            config.appId2,
            VerifierInfo(
                INexusVerifierWrapper(address(verifierWrapper)),
                address(0) // this needs to be updatated after the from chain mailbox is deployed
            )
        );

        vm.stopBroadcast();
    }
}
