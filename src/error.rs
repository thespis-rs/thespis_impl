use crate::import::*;

/// Result which has a ThesErr as error type.
//
pub type ThesRes<T> = Result<T, ThesErr>;

/// Errors that can happen in thespis_impl.
//
#[ derive( Debug, Clone, PartialEq, Eq ) ]
//
pub enum ThesErr
{
	/// Either the actor panicked during processing of the message or the mailbox was dropped.
	/// Only returned when doing a `call`.
	//
	ActorStoppedBeforeResponse
	{
		/// The actor concerned by the error.
		//
		actor: String
	},


	/// You try to use a mailbox that is already closed.
	//
	MailboxClosed
	{
		/// The actor concerned by the error.
		//
		actor: String
	},


	// /// The mailbox cannot take more messages right now. This only happens on
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


	/// Failed to spawn the mailbox.
	//
	Spawn
	{
		/// The actor concerned by the error.
		//
		actor: String
	}
}


impl std::error::Error for ThesErr {}


impl fmt::Display for ThesErr
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match &self
		{
			ThesErr::ActorStoppedBeforeResponse{ actor } =>

				write!( f, "The mailbox was closed before the result of the computation got returned upon `call`. For actor: {}", actor ),

			ThesErr::MailboxClosed{ actor } =>

				write!( f, "You try to use a mailbox that is already closed. For actor: {}", actor ),

			ThesErr::Spawn{ actor } =>

				write!( f, "Failed to spawn the mailbox for actor: {}", actor ),
		}
	}
}
