use crate::import::*;

/// Result which has a ThesErr as error type.
//
pub type ThesRes<T> = Result<T, ThesErr>;

/// Errors that can happen in thespis_impl.
//
#[ derive( Debug, Clone, Error, PartialEq, Eq ) ]
//
pub enum ThesErr
{
	/// You try to use a mailbox that is already closed.
	//
	#[ error( "You try to use a mailbox that is already closed. For actor: {actor}" ) ]
	//
	MailboxClosed
	{
		/// The actor concerned by the error.
		//
		actor: String
	},


	// /// The mailbox cannot take more messages right now. TODO: this only happens on
	// /// try_send on a future channel. For the moment there is no try_send in thespis.
	// //
	// #[ error( "The mailbox cannot take more messages right now. For actor: {actor}" ) ]
	// //
	// MailboxFull
	// {
	// 	/// The actor concerned by the error.
	// 	//
	// 	actor: String
	// },


	/// Either the actor panicked during processing of the message or the mailbox was dropped.
	/// Only returned when doing a `call`.
	//
	#[ error( "The mailbox was closed before the result of the computation got returned upon `call`. For actor: {actor}" ) ]
	//
	ActorStoppedBeforeResponse
	{
		/// The actor concerned by the error.
		//
		actor: String
	},


	/// Failed to spawn the mailbox.
	//
	#[ error( "Failed to spawn the mailbox for actor: {actor}" ) ]
	//
	Spawn
	{
		/// The actor concerned by the error.
		//
		actor: String

		// /// The underlying error.
		// source: anyhow::Error ,
	}
}
