use ethers::prelude::abigen;

// run `cargo doc -p contract-interface --open` to read struct definition

abigen!(
    NrhvFlow,
    "../../neurahive-contracts/artifacts/contracts/dataFlow/Flow.sol/Flow.json"
);

abigen!(
    PoraMine,
    "../../neurahive-contracts/artifacts/contracts/test/PoraMineTest.sol/PoraMineTest.json"
);
