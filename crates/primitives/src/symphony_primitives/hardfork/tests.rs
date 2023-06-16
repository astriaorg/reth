use crate::{Chain, Genesis};
use std::collections::BTreeMap;

// #[test]
// fn check_nonexistent_hardfork_from_str() {
//     assert!(Hardfork::from_str("not a hardfork").is_err());
// }

// #[test]
// fn check_fork_id_chainspec_with_fork_condition_never() {
//     let spec = ChainSpec {
//         chain: Chain::mainnet(),
//         genesis: Genesis::default(),
//         genesis_hash: None,
//         hardforks: BTreeMap::from([(Hardfork::Frontier, ForkCondition::Never)]),
//         fork_timestamps: Default::default(),
//         paris_block_and_final_difficulty: None,
//     };

//     assert_eq!(Hardfork::Frontier.fork_id(&spec), None);
// }

// #[test]
// fn check_fork_filter_chainspec_with_fork_condition_never() {
//     let spec = ChainSpec {
//         chain: Chain::mainnet(),
//         genesis: Genesis::default(),
//         genesis_hash: None,
//         hardforks: BTreeMap::from([(Hardfork::Shanghai, ForkCondition::Never)]),
//         fork_timestamps: Default::default(),
//         paris_block_and_final_difficulty: None,
//     };

//     assert_eq!(Hardfork::Shanghai.fork_filter(&spec), None);
// }

// #[test]
// fn check_hardfork_from_str() {
//     let hardfork_str = [
//         "frOntier",
//         "homEstead",
//         "dao",
//         "tAngerIne",
//         "spurIousdrAgon",
//         "byzAntium",
//         "constantinople",
//         "petersburg",
//         "istanbul",
//         "muirglacier",
//         "bErlin",
//         "lonDon",
//         "arrowglacier",
//         "grayglacier",
//         "PARIS",
//         "ShAnGhAI",
//     ];
//     let expected_hardforks = [
//         Hardfork::Frontier,
//         Hardfork::Homestead,
//         Hardfork::Dao,
//         Hardfork::Tangerine,
//         Hardfork::SpuriousDragon,
//         Hardfork::Byzantium,
//         Hardfork::Constantinople,
//         Hardfork::Petersburg,
//         Hardfork::Istanbul,
//         Hardfork::MuirGlacier,
//         Hardfork::Berlin,
//         Hardfork::London,
//         Hardfork::ArrowGlacier,
//         Hardfork::GrayGlacier,
//         Hardfork::Paris,
//         Hardfork::Shanghai,
//     ];

//     let hardforks: Vec<Hardfork> =
//         hardfork_str.iter().map(|h| Hardfork::from_str(h).unwrap()).collect();

//     assert_eq!(hardforks, expected_hardforks);
// }