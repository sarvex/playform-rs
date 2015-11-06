use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::sync::Mutex;
use stopwatch;
use thread_scoped;

use common::protocol;
use common::voxel;

use client;
use server;
use update_thread::update_thread;
use view_thread::view_thread;

// TODO: This is duplicated in the server. Fix that.
fn try_recv<T>(recv: &Receiver<T>) -> Option<T>
  where T: Send,
{
  match recv.try_recv() {
    Ok(msg) => Some(msg),
    Err(TryRecvError::Empty) => None,
    e => Some(e.unwrap()),
  }
}

#[allow(missing_docs)]
pub fn run(listen_url: &str, server_url: &str) {
  let (terrain_blocks_send, mut terrain_blocks_recv) = channel();
  let (view_thread_send0, mut view_thread_recv0) = channel();
  let (view_thread_send1, mut view_thread_recv1) = channel();

  let terrain_blocks_send: &Sender<(voxel::T, voxel::bounds::T)> = &terrain_blocks_send;
  let terrain_blocks_recv = &mut terrain_blocks_recv;
  let view_thread_send0 = &view_thread_send0;
  let view_thread_recv0 = &mut view_thread_recv0;
  let view_thread_send1 = &view_thread_send1;
  let view_thread_recv1 = &mut view_thread_recv1;

  let quit = Mutex::new(false);
  let quit = &quit;

  let server = server::new(&server_url, &listen_url);

  let client = connect_client(&listen_url, &server);
  let client = &client;

  {
    let update_thread = {
      let client = &client;
      let view_thread_send0 = view_thread_send0.clone();
      let view_thread_send1 = view_thread_send1.clone();
      let server = server.clone();
      let terrain_blocks_send = terrain_blocks_send.clone();
      unsafe {
        thread_scoped::scoped(move || {
          update_thread(
            quit,
            client,
            &mut || { server.listen.try() },
            &mut || { try_recv(terrain_blocks_recv) },
            &mut |up| { view_thread_send0.send(up).unwrap() },
            &mut |up| { view_thread_send1.send(up).unwrap() },
  	        &mut |up| { server.talk.tell(&up) },
            &mut |voxel, bounds| { terrain_blocks_send.send((voxel, bounds)).unwrap() },
          );

          stopwatch::clone()
        })
      }
    };

    {
      let server = server.clone();
      view_thread(
        client.player_id,
        &mut || {
          match view_thread_recv0.try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) =>
              panic!("view_thread_send should not be closed."),
          }
        },
        &mut || {
          match view_thread_recv1.try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) =>
              panic!("view_thread_send should not be closed."),
          }
        },
        &mut |server_update| { server.talk.tell(&server_update) },
      );

      stopwatch::clone().print();
    }

    // View thread returned, so we got a quit event.
    *quit.lock().unwrap() = true;

    let stopwatch = update_thread.join();
    stopwatch.print();
  }
}

fn connect_client(listen_url: &str, server: &server::T) -> client::T {
  // TODO: Consider using RPCs to solidify the request-response patterns.
  server.talk.tell(&protocol::ClientToServer::Init(listen_url.to_owned()));
  loop {
    match server.listen.wait() {
      protocol::ServerToClient::LeaseId(client_id) => {
        server.talk.tell(&protocol::ClientToServer::AddPlayer(client_id));
        let client_id = client_id;
        loop {
          match server.listen.wait() {
            protocol::ServerToClient::PlayerAdded(player_id, position) => {
              return client::new(client_id, player_id, position);
            },
            msg => {
              // Ignore other messages in the meantime.
              warn!("Ignoring: {:?}", msg);
            },
          }
        }
      },
      msg => {
        // Ignore other messages in the meantime.
        warn!("Ignoring: {:?}", msg);
      },
    }
  }
}
