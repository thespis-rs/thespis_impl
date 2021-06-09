use crate::{ import::*, ActorInfo };

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
	ActorStoppedBeforeResponse( Arc<ActorInfo> ),


	/// You try to use a mailbox that is already closed. The mailbox can be closed by dropping all
	/// strong addresses to it or by dropping the future that is running it.
	///
	/// When you get this error, the mailbox is gone and the address should be dropped. It will never
	/// accept messages again.
	//
	MailboxClosed( Arc<ActorInfo> ),


	// /// The mailbox cannot take more messages right now. This only happens on
	// /// try_send on a future channel. For the moment there is no try_send in thespis.
	// //
	// #[ error( "The mailbox cannot take more messages right now. For actor: {actor}" ) ]
	// //
	// MailboxFull
	// {
	// 	/// The actor concerned by the error.
	// 	//
	// 	actor: ActorInfo
	// },


	/// Failed to spawn the mailbox.
	//
	Spawn( Arc<ActorInfo> )
}


impl std::error::Error for ThesErr {}


impl fmt::Display for ThesErr
{
	fn fmt( &self, f: &mut fmt::Formatter<'_> ) -> fmt::Result
	{
		match &self
		{
			ThesErr::ActorStoppedBeforeResponse(actor) =>

				write!( f, "The mailbox was closed before the result of the computation got returned upon `call`. For actor: {}", actor ),

			ThesErr::MailboxClosed(actor) =>

				write!( f, "You try to use a mailbox that is already closed. For actor: {}", actor ),

			ThesErr::Spawn(actor) =>

				write!( f, "Failed to spawn the mailbox for actor: {}", actor ),
		}
	}
}
