pub mod actors;

#[ cfg( feature = "remote" ) ] pub mod remote;

pub mod import
{
	pub use
	{
		futures       :: { future::{ FutureExt } } ,
		thespis       :: { * } ,
		log           :: { * } ,
	};

	#[ cfg( feature = "remote" ) ]
	//
	pub use
	{
		tokio        :: { await as await01, prelude::{ StreamAsyncExt, Stream as TokStream, stream::SplitStream as TokSplitStream, stream::SplitSink as TokSplitSink }, net::{ TcpStream, TcpListener }, codec::{ Decoder, Framed } } ,

		thespis_impl:: { remote::*, single_thread::* } ,
		std          :: { net::SocketAddr            } ,

	};
}
