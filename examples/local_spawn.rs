#![feature(arbitrary_self_types, specialization, nll, never_type, unboxed_closures, trait_alias)]

#![allow(dead_code, unused_imports)]


use
{
    futures::{future::{Future, FutureExt}, task::{LocalSpawn, SpawnExt, LocalSpawnExt}, executor::LocalPool},
    std::{pin::Pin},
    log::{*},
    thespis::{*},
    thespis_impl::{*},
    async_runtime::rt,
};


#[derive(Actor)]
//
struct MyActor;

struct Ping(String);


impl Message for Ping
{
    type Return = String;
}


impl Handler<Ping> for MyActor
{
    fn handle(&mut self, _msg: Ping) -> Return<String> {
        Box::pin(async move
            {
                "pong".into()
            })
    }
}


fn main()
{
    let mut local_pool = futures::executor::LocalPool::new();
    let mut thread_pool = futures::executor::ThreadPool::new().unwrap();

    let mut addr_local = {
        let actor = MyActor;
        let inbox = Inbox::new();
        let addr = inbox.addr();
        local_pool.spawner().spawn_local(
            inbox.start_fut_local(actor)
        ).unwrap();
        addr
    };

    let mut addr_thread = {
        let actor = MyActor;
        let inbox = Inbox::new();
        let addr = inbox.addr();
        thread_pool.spawn(
            inbox.start_fut(actor)
        ).unwrap();
        addr
    };

    local_pool.spawner().spawn_local(async move {
        let result_local = addr_local.call(Ping("ping".into())).await.expect("Call failed");
        assert_eq!("pong".to_string(), result_local);
        dbg!(result_local);

        let result_thread = addr_thread.call(Ping("ping".into())).await.expect("Call failed");
        assert_eq!("pong".to_string(), result_thread);
        dbg!(result_thread);
    }).unwrap();

    local_pool.run();
}
