#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
use frame_support::sp_runtime::offchain::{http, Duration};
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			let _ = Self::fetch_n_parse().unwrap();
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type WeightInfo: WeightInfo;
		#[pallet::constant]
		type StringLimit: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored { something: u32, who: T::AccountId },
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
	struct HackerNewsInfo<T: Config> {
		by: BoundedVec<u8, <T as Config>::StringLimit>,
		title: BoundedVec<u8, <T as Config>::StringLimit>,
		url: BoundedVec<u8, <T as Config>::StringLimit>,
		descendants: u32,
	}


	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		HttpFetchingError,
		DeserializeToStrError,
		DeserializeToObjError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			Self::deposit_event(Event::SomethingStored { something, who });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			match <Something<T>>::get() {
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}

	const HTTP_REMOTE_REQUEST: &str = "https://hacker-news.firebaseio.com/v0/item/9129911.json";

	const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
	use scale_info::prelude::{string::String, vec::Vec};

	impl<T: Config> Pallet<T> {
		fn fetch_n_parse() -> Result<HackerNewsInfo<T>, Error<T>> {
			let resp_bytes = Self::fetch_from_remote().map_err(|e| {
				log::error!("fetch_from_remote error: {:?}", e);
				<Error<T>>::HttpFetchingError
			})?;


			let resp_strr =
				String::from_utf8(resp_bytes).map_err(|_| <Error<T>>::DeserializeToStrError)?;

			let resp_str = resp_strr.as_str();
			log::info!("fetch_n_parse: {}", resp_str);

			let info: HackerNewsInfo<T> =
				serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::DeserializeToObjError)?;
			
			let c = 
				codec::json

			Ok(info)
		}

		fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
			let request = http::Request::get(HTTP_REMOTE_REQUEST);

			let timeout =
				sp_io::offchain::timestamp().add(Duration::from_millis(FETCH_TIMEOUT_PERIOD));

			let pending = request
				.deadline(timeout) // Setting the timeout time
				.send() // Sending the request out by the host
				.map_err(|e| {
					log::error!("{:?}", e);
					<Error<T>>::HttpFetchingError
				})?;

			let response = pending
				.try_wait(timeout)
				.map_err(|e| {
					log::error!("{:?}", e);
					<Error<T>>::HttpFetchingError
				})?
				.map_err(|e| {
					log::error!("{:?}", e);
					<Error<T>>::HttpFetchingError
				})?;

			if response.code != 200 {
				log::error!("Unexpected http request status code: {}", response.code);
				return Err(<Error<T>>::HttpFetchingError);
			}

			Ok(response.body().collect::<Vec<u8>>())
		}
	}
}
