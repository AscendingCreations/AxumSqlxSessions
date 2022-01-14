use crate::sessions::SqlxSessionData;
use serde::de::DeserializeOwned;
use serde::Serialize;

//used to enfore Sqlxsessions binding in other libs.
pub trait SessionBind {
    fn tap<T: DeserializeOwned>(
        &self,
        func: impl FnOnce(&mut SqlxSessionData) -> Option<T>,
    ) -> Option<T>;

    ///Sets the Entire Session to be Cleaned on next load.
    fn destroy(&self);

    ///Used to get data stored within SessionDatas hashmap from a key value.
    fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T>;

    /// Used to Set data to SessionData via a Key and the Value to Set.
    fn set(&self, key: &str, value: impl Serialize);

    ///used to remove a key and its data from SessionData's Hashmap
    fn remove(&self, key: &str);

    /// Will instantly clear all data from SessionData's Hashmap
    fn clear_all(&self);

    /// Returns a Count of all Sessions currently within the Session Store.
    fn count(&self) -> i64;
}
