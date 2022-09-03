// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

#[derive(Copy, Clone)]
pub struct PartialProverSolution<N: Network> {
    pub address: Address<N>,
    pub nonce: u64,
    pub commitment: Commitment<N::PairingCurve>,
}

impl<N: Network> PartialProverSolution<N> {
    pub fn new(address: Address<N>, nonce: u64, commitment: Commitment<N::PairingCurve>) -> Self {
        Self { address, nonce, commitment }
    }

    pub fn address(&self) -> &Address<N> {
        &self.address
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn commitment(&self) -> &Commitment<N::PairingCurve> {
        &self.commitment
    }
}

impl<N: Network> Eq for PartialProverSolution<N> {}

impl<N: Network> PartialEq for PartialProverSolution<N> {
    /// Implements the `Eq` trait for the PartialProverSolution.
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.nonce == other.nonce && self.commitment == other.commitment
    }
}

// TODO (raychu86): Use derive Hash. It seems commitment and proof do not derive it properly.
impl<N: Network> core::hash::Hash for PartialProverSolution<N> {
    /// Implements the `Hash` trait for the PartialProverSolution.
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.nonce.hash(state);
        self.commitment.0.hash(state);
    }
}

impl<N: Network> ToBytes for PartialProverSolution<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.address.write_le(&mut writer)?;
        self.nonce.write_le(&mut writer)?;
        self.commitment.write_le(&mut writer)
    }
}

impl<N: Network> FromBytes for PartialProverSolution<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let address: Address<N> = FromBytes::read_le(&mut reader)?;
        let nonce = u64::read_le(&mut reader)?;
        let commitment = Commitment::read_le(&mut reader)?;

        Ok(Self { address, nonce, commitment })
    }
}

impl<N: Network> Serialize for PartialProverSolution<N> {
    /// Serializes the PartialProverSolution to a JSON-string or buffer.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match serializer.is_human_readable() {
            true => {
                let mut partial_prover_solution = serializer.serialize_struct("PartialProverSolution", 3)?;
                partial_prover_solution.serialize_field("address", &self.address)?;
                partial_prover_solution.serialize_field("nonce", &self.nonce)?;
                partial_prover_solution.serialize_field("commitment", &self.commitment.0)?;
                partial_prover_solution.end()
            }
            false => ToBytesSerializer::serialize_with_size_encoding(self, serializer),
        }
    }
}

impl<'de, N: Network> Deserialize<'de> for PartialProverSolution<N> {
    /// Deserializes the PartialProverSolution from a JSON-string or buffer.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match deserializer.is_human_readable() {
            true => {
                let partial_prover_solution = serde_json::Value::deserialize(deserializer)?;
                Ok(Self::new(
                    serde_json::from_value(partial_prover_solution["address"].clone()).map_err(de::Error::custom)?,
                    serde_json::from_value(partial_prover_solution["nonce"].clone()).map_err(de::Error::custom)?,
                    Commitment(
                        serde_json::from_value(partial_prover_solution["commitment"].clone())
                            .map_err(de::Error::custom)?,
                    ),
                ))
            }
            false => {
                FromBytesDeserializer::<Self>::deserialize_with_size_encoding(deserializer, "partial prover solution")
            }
        }
    }
}

impl<N: Network> FromStr for PartialProverSolution<N> {
    type Err = Error;

    /// Initializes the PartialProverSolution from a JSON-string.
    fn from_str(partial_prover_solution: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(partial_prover_solution)?)
    }
}

impl<N: Network> Debug for PartialProverSolution<N> {
    /// Prints the PartialProverSolution as a JSON-string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for PartialProverSolution<N> {
    /// Displays the PartialProverSolution as a JSON-string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).map_err::<fmt::Error, _>(ser::Error::custom)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console::{account::PrivateKey, network::Testnet3};

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_serde_json() -> Result<()> {
        let mut rng = TestRng::default();
        let private_key = PrivateKey::<CurrentNetwork>::new(&mut rng)?;
        let address = Address::try_from(private_key)?;

        // Sample a new partial prover solution.
        let expected = PartialProverSolution::new(address, u64::rand(&mut rng), Commitment(rng.gen()));

        // Serialize
        let expected_string = &expected.to_string();
        let candidate_string = serde_json::to_string(&expected)?;
        assert_eq!(expected, serde_json::from_str(&candidate_string)?);

        // Deserialize
        assert_eq!(expected, PartialProverSolution::from_str(expected_string)?);
        assert_eq!(expected, serde_json::from_str(&candidate_string)?);

        Ok(())
    }

    #[test]
    fn test_bincode() -> Result<()> {
        let mut rng = TestRng::default();
        let private_key = PrivateKey::<CurrentNetwork>::new(&mut rng)?;
        let address = Address::try_from(private_key)?;

        // Sample a new partial prover solution.
        let expected = PartialProverSolution::new(address, u64::rand(&mut rng), Commitment(rng.gen()));

        // Serialize
        let expected_bytes = expected.to_bytes_le()?;
        let expected_bytes_with_size_encoding = bincode::serialize(&expected)?;
        assert_eq!(&expected_bytes[..], &expected_bytes_with_size_encoding[8..]);

        // Deserialize
        assert_eq!(expected, PartialProverSolution::read_le(&expected_bytes[..])?);
        assert_eq!(expected, bincode::deserialize(&expected_bytes_with_size_encoding[..])?);

        Ok(())
    }
}