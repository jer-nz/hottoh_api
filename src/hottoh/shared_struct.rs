use crate::hottoh::hottoh_structs::{DAT0Data, DAT1Data, DAT2Data, INFData};
use serde::Serialize;

/// Shared state containing all data from the stove
///
/// This structure holds all the data retrieved from the stove,
/// including general information, status, temperatures, and settings.
#[derive(Debug, Serialize, Default, Clone)]
pub struct SharedState {
    /// General information about the stove
    inf: INFData,
    /// Main stove data (status, temperatures, power levels, etc.)
    dat0: DAT0Data,
    /// Additional stove data (temperatures)
    dat1: DAT1Data,
    /// Additional stove data (pumps, valves, etc.)
    dat2: DAT2Data,
}

impl SharedState {
    /// Creates a new SharedState with default values
    ///
    /// # Returns
    ///
    /// * `SharedState` - A new instance with default values
    pub fn new() -> Self {
        Self {
            inf: INFData::default(),
            dat0: DAT0Data::default(),
            dat1: DAT1Data::default(),
            dat2: DAT2Data::default(),
        }
    }

    /// Gets the general information data
    ///
    /// # Returns
    ///
    /// * `&INFData` - Reference to the INF data
    pub fn get_inf(&self) -> &INFData {
        &self.inf
    }

    /// Gets the main stove data
    ///
    /// # Returns
    ///
    /// * `&DAT0Data` - Reference to the DAT0 data
    pub fn get_dat0(&self) -> &DAT0Data {
        &self.dat0
    }

    /// Gets the additional temperature data
    ///
    /// # Returns
    ///
    /// * `&DAT1Data` - Reference to the DAT1 data
    pub fn get_dat1(&self) -> &DAT1Data {
        &self.dat1
    }

    /// Gets the additional pump and valve data
    ///
    /// # Returns
    ///
    /// * `&DAT2Data` - Reference to the DAT2 data
    pub fn get_dat2(&self) -> &DAT2Data {
        &self.dat2
    }

    /// Updates the general information data
    ///
    /// # Arguments
    ///
    /// * `inf` - The new INF data
    pub fn set_inf(&mut self, inf: &INFData) {
        self.inf = inf.clone();
    }

    /// Updates the main stove data
    ///
    /// # Arguments
    ///
    /// * `dat0` - The new DAT0 data
    pub fn set_dat0(&mut self, dat0: &DAT0Data) {
        self.dat0 = dat0.clone();
    }

    /// Updates the additional temperature data
    ///
    /// # Arguments
    ///
    /// * `dat1` - The new DAT1 data
    pub fn set_dat1(&mut self, dat1: &DAT1Data) {
        self.dat1 = dat1.clone();
    }

    /// Updates the additional pump and valve data
    ///
    /// # Arguments
    ///
    /// * `dat2` - The new DAT2 data
    pub fn set_dat2(&mut self, dat2: &DAT2Data) {
        self.dat2 = dat2.clone();
    }
}
