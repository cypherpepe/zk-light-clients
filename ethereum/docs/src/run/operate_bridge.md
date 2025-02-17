 # Operate the bridge
 
In the previous sections we have gone over the steps to setup each components that are
available in the source of the repository so that they can start interacting with each
other. 

However, in a practical scenario, the client and verifier contracts will have
to be adapted to the use case that a user wants to implement. We will go over the
steps to adapt the components for any use case in this section.

## Adapt the client

### Initialize the client

Before we can start fetching data from the Ethereum network, we need to initialize
the client. To do so we have to leverage the checkpoint mechanism available for
the sync protocol. The logic to initialize the client is quite straight forward and an 
example implementation can be found [in our mock client](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/light-client/src/bin/client.rs#L256-L372).

### Fetch Merkle Proof data

The first piece that will need some refactoring to adapt to a new use case is the client.
The client should be considered as the main entry point for a bridge, and is responsible 
for fetching data from the Ethereum network and submitting it to the prover.

The first thing to note is that the data to be fetched from the Ethereum network 
will differ depending on how a liquidity provider handles its assets. However, most assets
will be living in a smart contract on Ethereum at a given storage key, making our
current implementation that leverages [the EIP-1186](https://eips.ethereum.org/EIPS/eip-1186)
flexible to most use cases.

The EIP-1186 has two main arguments that need to be passed, the address of 
the targeted contract and the storage key.

The address of the targeted contract can be easily retrieved from an explorer of 
the chain. The storage key you are looking for is a bit more tricky to compute.To
understand which storage key you will want to use as a pointer to the data 
to bridge, refer to [the Ethereum documentation on "Layout of State Variables in Storage and Transient Storage"](https://docs.soliditylang.org/en/v0.8.28/internals/layout_in_storage.html#index-0).

Once we have a hold on the storage key we can use it as a parameter to call
the `eth_getProof` RPC endpoint, thus fetching a Merkle Proof for the inclusion of
the data for the storage key at a given block height on the Ethereum Network.

The payload we will get from the `eth_getProof` RPC endpoint will be used to generate
a proof that can be verified on-chain. The proof will be generated by the prover.

In the mock client we have developed to showcase the bridge, we can actually find 
the values for the smart contract address and the storage key being [declared as 
constants](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/light-client/src/bin/client.rs#L30-L37) in the Rust code base.

### Transform the Merkle Proof data

In our codebase the structure representing the fetched Merkle Proof data is
[`GetProofResponse`](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/light-client/src/types/storage.rs#L5-L12).
This data has to be transformed in the inner type used by the prover, [`EIP1186Proof`](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/core/src/merkle/storage_proofs.rs#L42-L51).
An example of this data transformation implementation can be found [in the codebase](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/core/src/merkle/storage_proofs.rs#L53-L77).

## Run the prover

The prover is quite straight forward to run. When ran in `single` mode, the only
parameter to properly set is the address it should listen to for incoming request.

It consists of a lightweight router the will listen to the following routes:
-  (**GET**) `/health`: Operationnal endpoint the returns a 200 HTTP code when the server is ready to receive requests
- (**GET**) `/ready`: Operationnal endpoint the returns a 200 HTTP code when the server is not currently handling a 
  request
- (**POST**) `/inclusion/proof`: Endpoint to submit a proof request for an inclusion proof
- (**POST**) `/inclusion/verify`: Endpoint to submit a proof request for an inclusion proof verification
- (**POST**) `/committee/proof`: Endpoint to submit a proof request for a committee proof
- (**POST**) `/committee/verify`: Endpoint to submit a proof request for a committee proof verification

For proofs related endpoint the payload is a binary serialized payload that is sent over
HTTP. The Rust type in our codebase representing such types is [`Request`](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/light-client/src/types/network.rs#L6-L16).

The bytes payload format is the following:

**Proof generation**

| Name         | Byte offset | Description                                                                                                              |
|--------------|-------------|--------------------------------------------------------------------------------------------------------------------------|
| Request type | 0           | Type of the request payload                                                                                              |
| Proving mode | 1           | Type of the proof that the proof server should generate. `0` for STARK and `1` for SNARK                                 |
| Proof inputs | 2           | SSZ encoded inputs for the proof generation. Serialized [`StorageInclusionIn`](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/light-client/src/proofs/inclusion.rs#L61-L67) for inclusion and serialized [`CommitteeChangeIn`](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/light-client/src/proofs/committee_change.rs#L59-L64) for committee change. |

**Proof verification**

| Name        | Byte offset | Description                                                                                                                                                                              |
|-------------|-------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Request type | 0           | Type of the request payload                                                                                                                                                              |
| Proof type  | 1           | Type of the proof that the payload contains. `0` for STARK and `1` for SNARK                                                                                                             |
| Proof | 2           | Bytes representing a JSON serialized [`SphinxProofWithPublicValues`](https://github.com/argumentcomputer/sphinx/blob/36f3f9072dc187612640e2725a2f7524cf2f2215/sdk/src/proof.rs#L21-L28). |

The response bodies are more straight forward:

**Proof generation**

| Name | Byte offset | Description |
|------|-------------|-------------|
| Proof type  | 0           | Type of the proof that the payload contains. `0` for STARK and `1` for SNARK                                                                                                             |
| Proof | 1           | Bytes representing a JSON serialized [`SphinxProofWithPublicValues`](https://github.com/argumentcomputer/sphinx/blob/36f3f9072dc187612640e2725a2f7524cf2f2215/sdk/src/proof.rs#L21-L28). |

**Proof verification**

| Name                          | Byte offset | Description                                                                                |
|-------------------------------|-------------|--------------------------------------------------------------------------------------------|
| Successful proof verification | 0           | A `0` (fail) or `1` (success) byte value representing the success of a proof verification. |

## Adapt the verifier

In the following section we will touch upon how a verifier contract has to be updated
depending on a use case. However, it has to be kept in mind that some core
data will have to be passed even thought some modifications have to be done 
for different use cases.

### Core data

> **Note**
> 
> The following documentation will be for SNARK proofs, as they are the only 
> proofs that can be verified on our home chains.

The core data to be passed to any verification contrtact are the following:
- Verifying key: A unique key represented as 32 bytes, related to the program that is meant to be verified
- Public values: Serialized public values of the proof
- Proof: The serialized proof to be verified

**Verifying key**

The verifying key for a program at a given commit can be found in its fixture file
in the format of a hexified string prefixed by `0x`. There is [one file for the committee
change](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/pact/fixtures/epoch_change_fixture.json) 
program and one file for [the inclusion program](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/pact/fixtures/inclusion_fixture.json).

**Public values**

The public values and serialized proof data can be found through the type [`SphinxProofWithPublicValues`](https://github.com/argumentcomputer/sphinx/blob/36f3f9072dc187612640e2725a2f7524cf2f2215/sdk/src/proof.rs#L21-L28)
returned as an HTTP response body by the prover. 

The public values can be found under the `public_values` property and are already
represented as a `Buffer` which data are to be transmitted to the verifier contract.
In the fixture files we leverage in our codebase, the public values are represented
as a hexified string prefixed by `0x`.

**Proof**

The proof data to be passed to the verifier contract will depend on the chain
hosting it. The difference emerges from how the verification is ran on the network.
On the one hand, if the verification is ran natively via an smart contract FFI call for example, then
the proof will have to not be encoded and be passed in its raw form. On the other
hand, if the verification is directly done in a smart contract, then the proof will
have to be passed in its encoded format.

In our case we have showcased the serialization format for two chains,
Aptos for the smart contract verification and Kadena for the native verification.

For Aptos, the proof data is an array of bytes with the following format:

| Name                 | Byte offset | Description                                                                                                                                                                                                                                                                          |
|----------------------|-------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Verifying key prefix | 0           | Prefix to the encoded proof, a 4 bytes value corresponding to the first 4 bytes of the verifying key.                                                                                                                                                                                |
| Encoded proof        | 4           | Encoded proof which value can be found in the returned SNARK proof from the prover represented as [`SphinxProofWithPublicValues`](https://github.com/argumentcomputer/sphinx/blob/36f3f9072dc187612640e2725a2f7524cf2f2215/sdk/src/proof.rs#L21-L28) under [`proof.encoded_proof`](https://github.com/argumentcomputer/sphinx/blob/dev/recursion/gnark-ffi/src/plonk_bn254.rs#L24) |

For Kadena, the proof data is an array of bytes with the following format:

| Name      | Byte offset | Description                                                                                                                                                                                                                                                                                                                                                                |
|-----------|-------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Raw proof | 0           | Raw proof which value can be found in the returned SNARK proof from the prover represented as [`SphinxProofWithPublicValues`](https://github.com/argumentcomputer/sphinx/blob/36f3f9072dc187612640e2725a2f7524cf2f2215/sdk/src/proof.rs#L21-L28) under [`proof.raw_proof`](https://github.com/argumentcomputer/sphinx/blob/dev/recursion/gnark-ffi/src/plonk_bn254.rs#L25) |

Example of the proof data extraction can be found for both [Aptos](https://github.com/argumentcomputer/zk-light-clients/blob/dev/fixture-generator/src/bin/main.rs#L221) 
and [Kadena](https://github.com/argumentcomputer/zk-light-clients/blob/dev/fixture-generator/src/bin/main.rs#L283) in
our fixture generation crate.

### Wrapper logic

The wrapper logic refers to a smart contract wrapping the proof verification logic
with the use case specific logic. It is needed to ensure that the verified proof corresponds
to the expected data.

The logic to be executed in the wrapper contract will depend on the use case. However,
there are some core logic that have to be executed for the inclusion and committee
change proof verification. The logic that has to be kept for the inclusion verification
and the committee change program are showcased in both our Move contracts ([inclusion](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/move/sources/wrapper.move#L146-L166) and [commitee change](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/move/sources/wrapper.move#L101-L115)).

The place where a user can add its own use case logic is where we currently print out some values
in the Move contracts ([inclusion](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/move/sources/wrapper.move#L167-L211) and [committee change](https://github.com/argumentcomputer/zk-light-clients/blob/dev/ethereum/move/sources/wrapper.move#L122-L123)).