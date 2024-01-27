use aptos_types::account_config::resources;
use ethers_core::k256::sha2::digest::typenum::Mod;
use move_core_types::{effects::{ChangeSet, Op}, language_storage::ModuleId};
use sov_state::{WorkingSet, StateMap};
use sov_movevm_types::identifiers::AccessPathWrapper;
use std::cell::RefCell;
use aptos_types::access_path::AccessPath;

/// This is an implmenetantion of remote cache which will be back of TransactionDataCache
/// review? I used RefCell but not sure if it can be secure.
pub struct ChangeSetPublisher<'a, C: sov_modules_api::Context> {
  pub remote_cache: sov_state::StateMap<AccessPathWrapper, Vec<u8>>,
  pub working_set: RefCell<&'a mut WorkingSet<C::Storage>>,
}

impl<'a, C: sov_modules_api::Context> ChangeSetPublisher<'a, C> {
  pub fn new(
      remote_cache: StateMap<AccessPathWrapper, Vec<u8>>,
      working_set: &'a mut WorkingSet<C::Storage>,
  ) -> Self {
      Self {
          remote_cache,
          working_set: RefCell::new(working_set),
      }
  }

  pub fn publish(&self, change_set: ChangeSet) -> Result<(), anyhow::Error> {

    let mut working_set = self.working_set.borrow_mut();
    for (account_address, account_changes) in change_set.accounts().iter() {

      let (modules, resources) = account_changes.to_owned().into_inner(); // owning better than cloning

      // modules
      for (path, op) in modules.iter() {
        let ap = AccessPath::code_access_path(ModuleId::new(
          account_address.clone(),
          path.clone()
        ));
        match op {
          Op::New(data) => {
            self.remote_cache.set(&AccessPathWrapper::new(ap), data, &mut working_set);
          },
          Op::Modify(data) => {
            self.remote_cache.set(&AccessPathWrapper::new(ap), data, &mut working_set);
          },
          Op::Delete => {
            self.remote_cache.remove(&AccessPathWrapper::new(ap),  &mut working_set);
          },
        }
      }

      // resources
      for (path, op) in resources.iter() {
        let ap = AccessPath::resource_access_path(*account_address, path.clone().to_owned())?; // struct tag is actually somewhat large so avoiding clone may be useful
        match op {
          Op::New(data) => {
            self.remote_cache.set(&AccessPathWrapper::new(ap), data, &mut working_set);
          },
          Op::Modify(data) => {
            self.remote_cache.set(&AccessPathWrapper::new(ap), data, &mut working_set);
          },
          Op::Delete => {
            self.remote_cache.remove(&AccessPathWrapper::new(ap),  &mut working_set);
          },
        }
      }

    }

    Ok(())

  }
}
