use actix_web::{error::InternalError, http::StatusCode, Error, HttpRequest, HttpResponse};
use futures::prelude::*;
use storage::{Storage, StorageError};

pub mod post;

type Request = HttpRequest<State>;
type AsyncResponse = Box<Future<Item = HttpResponse, Error = Error>>;

#[derive(Default)]
pub struct State {
	storage: Storage,
}

impl Into<InternalError<StorageError>> for StorageError {
	fn into(self) -> InternalError<StorageError> {
		InternalError::new(self, StatusCode::INTERNAL_SERVER_ERROR)
	}
}

impl Into<Error> for StorageError {
	fn into(self) -> Error {
		let ie: InternalError<_> = self.into();
		ie.into()
	}
}
