use futures::StreamExt;
//use subxt::ext::sp_runtime::DispatchErrorWithPostInfo;
//use subxt::ext::sp_runtime::AccountId32;
use subxt::utils::AccountId32;
use subxt::{tx::TxStatus,tx::PairSigner, OnlineClient, PolkadotConfig};
use sp_keyring::AccountKeyring;
use sp_keyring::sr25519::sr25519::Pair;
use thiserror::Error as ThisError;

//pub use crate::polkadot::runtime_types::pallet_pet::pallet;
//use subxt::error::DispatchError::Module;

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "/mnt/hddisk1/github/test-subxt/metadata.scale")]
pub mod polkadot {}
type PetId = u32;
type PetSpecies = polkadot::runtime_types::pallet_pet::pallet::Species;
type PetInfo = polkadot::runtime_types::pallet_pet::pallet::PetInfo;
type Error = polkadot::runtime_types::pallet_pet::pallet::Error;
type PetName = polkadot::runtime_types::bounded_collections::bounded_vec::BoundedVec<u8>;

#[derive(ThisError, Debug)]
pub enum PetError {
    #[error("this is no pet")]
    NoPetExists,
    #[error("unknown data store error")]
    Unknown,
}


async fn fetch_storage(
    api:&OnlineClient<PolkadotConfig>,
    send:&AccountId32,)
    -> Result<(PetId,PetInfo), Box<dyn std::error::Error>> {

    let storage_query = polkadot::storage().pet_module().pets_info(send);

    // Use that query to `fetch` a result. This returns an `Option<_>`, which will be
    // `None` if no value exists at the given address. You can also use `fetch_default`
    // where applicable, which will return the default value if none exists.
    let result = api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?;

    match result {
        Some(_) => {
            let (id,petinfo) = result.unwrap(); 
            println!("Pet id is {id:?}, pet infor is {petinfo:?}.");
            return Ok((id,petinfo));
        },
        None => {
            println!("Sorry, you don't have a pet.");
            return Err(PetError::NoPetExists.into());
        },
    }
    
}


async fn transfer(
    api:&OnlineClient<PolkadotConfig>,
    from:&PairSigner<PolkadotConfig,Pair>,
    receiver:AccountId32,
    petid:PetId) 
    -> Result<(), Box<dyn std::error::Error>> {
    
    //Build a pet transfer extrinsic.
    let pet_transfer_tx = polkadot::tx().pet_module().transfer_pet(receiver, petid);

    let mut transfer_pet = api
        .tx()
        .sign_and_submit_then_watch_default(&pet_transfer_tx, from)
        .await?;
    
    while let Some(status) = transfer_pet.next().await {
            match status? {
                // It's finalized in a block!
                TxStatus::Finalized(in_block) => {
                    println!(
                        "Transaction is finalized in block ",
                        //in_block.extrinsic_hash(),
                        //in_block.block_hash()
                    );
                    
                    // grab the events and fail if no ExtrinsicSuccess event seen:
                    let events = in_block.fetch_events().await?;
                    // We can look for events (this uses the static interface; we can also iterate
                    //over them and dynamically decode them):
                    let transfer_event = events.find_first::<polkadot::pet_module::events::PetTransfered>()?;
                    //let transfer_event = events.find_first::<ExtrinsicFailed>()?;
                    if let Some(_) = transfer_event {
                        println!("Pet saled!");
                    } else {
                        println!("Error::SomethingWrong");
                    }
                }
                TxStatus::Ready => {}
                TxStatus::InBlock(_) => {}
                // Just log any other status we encounter:
                other => {
                    println!("Status: {other:?}");
                }
            }
        }

    Ok(())
}
async fn mint (api:&OnlineClient<PolkadotConfig>, 
    petid:PetId, 
    species:PetSpecies, 
    name:PetName,
    from:&PairSigner<PolkadotConfig,Pair>)
    -> Result<(), Box<dyn std::error::Error>> {
    
    // Build a pet mint extrinsic.
    let balance_transfer_tx = polkadot::tx().pet_module().mint_pet(name,species,petid);
    // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
    // and in a finalized block. We get back the extrinsic events if all is well.
    
    let mut mint_pet = api
         .tx()
         .sign_and_submit_then_watch_default(&balance_transfer_tx, from)
         //.await?
         //.wait_for_finalized_success()
         .await?;

    while let Some(status) = mint_pet.next().await {
            match status? {
                // It's finalized in a block!
                TxStatus::Finalized(in_block) => {
                    println!(
                        "Transaction is finalized in block ",
                        //in_block.extrinsic_hash(),
                        //in_block.block_hash()
                    );
                    
                    // grab the events and fail if no ExtrinsicSuccess event seen:
                    let events = in_block.fetch_events().await?;
                    
                    //println!("Event:{events:?}");
                    // We can look for events (this uses the static interface; we can also iterate
                    //over them and dynamically decode them):
                    let transfer_event = events.find_first::<polkadot::pet_module::events::PetMinted>()?;
                    let error_event = events.find_first::<polkadot::system::events::ExtrinsicFailed>()?;

                    if let Some(_event) = transfer_event {
                        println!("Yeah! You have your own pet!");
                    } else {
                        println!("Error::AlreadyHavePet");
                    }
                }
                TxStatus::Ready => {}
                TxStatus::InBlock(_) => {}
                // Just log any other status we encounter:
                other => {
                    println!("Status: {other:?}");
                }
            }
        }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Create a new API client, configured to talk to Polkadot nodes.
    let api = OnlineClient::<PolkadotConfig>::new().await?;

    //Some pet information, include petname, species, petid
    let petid : PetId = 1;
    let species = polkadot::runtime_types::pallet_pet::pallet::Species::Turtle;
    let name = "Annatle".to_string().into_bytes();
    let petname = polkadot::runtime_types::bounded_collections::bounded_vec::BoundedVec(name);
    
    //Mint a pet for account Alice.
    let from = PairSigner::new(AccountKeyring::Alice.pair());
    mint(&api,petid,species,petname,&from).await?;

    //Transfer Alice's pet to Bob.
    let dest: AccountId32 = AccountKeyring::Bob.to_account_id().into();
    transfer(&api,&from,dest,petid).await?;
    
    //Find information about Bob's pet.
    let send: AccountId32= AccountKeyring::Alice.to_account_id().into();
    fetch_storage(&api, &send).await?;

    Ok(())

}