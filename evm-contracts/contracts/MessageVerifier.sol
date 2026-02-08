// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;


contract MessageVerifier {

    // Wormhole VAA domain prefix
    bytes1 private constant VAA_PREFIX = 0x01;



    function hashVAA(
        uint8 version,
        uint32 guardianSetIndex,
        uint16 emitterChainId,
        bytes32 emitterAddress,
        uint64 sequence,
        bytes calldata payload
    ) public pure returns (bytes32) {
        return keccak256(
            abi.encodePacked(
                VAA_PREFIX,
                version,
                guardianSetIndex,
                emitterChainId,
                emitterAddress,
                sequence,
                payload
            )
        );
    }



    function hashPayload(bytes calldata payload) external pure returns (bytes32) {
        return keccak256(payload);
    }
}
