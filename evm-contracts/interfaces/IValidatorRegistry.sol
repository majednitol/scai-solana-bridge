// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IValidatorRegistry {
    function verifySignatures(bytes32 hash, bytes[] calldata signatures) external view returns (bool);
}
