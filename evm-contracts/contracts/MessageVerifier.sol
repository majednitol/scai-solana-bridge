// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./ValidatorRegistry.sol";

interface IValidatorRegistry {
    function verifySignatures(bytes32 messageHash, bytes[] calldata signatures) external view returns (bool);
}
contract MessageVerifier {
    IValidatorRegistry public validatorRegistry;

    constructor(address _validatorRegistry) {
        require(_validatorRegistry != address(0), "MessageVerifier: zero registry");
        validatorRegistry = IValidatorRegistry(_validatorRegistry);
    }

    function hashMessage(
        uint256 chainId,
        address recipient,
        uint256 amount,
        bytes32 orderId,
        uint256 nonce,
        uint256 timestamp
    ) public pure returns (bytes32) {
        bytes32 rawHash = keccak256(
            abi.encodePacked(chainId, recipient, amount, orderId, nonce, timestamp)
        );

        return keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", rawHash)
        );
    }

    function verifySignatures(bytes32 message, bytes[] calldata signatures) external view returns (bool) {
        return validatorRegistry.verifySignatures(message, signatures);
    }
}
