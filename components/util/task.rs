/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::str::IntoMaybeOwned;
use std::task;
use std::comm::Sender;
use std::task::TaskBuilder;
use native::task::NativeTaskBuilder;

pub fn spawn_named<S: 'static + Send + IntoMaybeOwned<'static> + Clone + fmt::Show>(name: S, f: proc():Send) {
    let name_clone = name.clone();
    let builder = task::TaskBuilder::new().named(name);
    builder.spawn(proc() {
        f();
        error!("{} is dead", name_clone);
    });
}

pub fn spawn_named_native<S: 'static + Send + IntoMaybeOwned<'static> + Clone + fmt::Show>(name: S, f: proc():Send) {
    let name_clone = name.clone();
    let builder = task::TaskBuilder::new().native().named(name);
    builder.spawn(proc() {
        f();
        error!("{} is dead", name_clone);
    });
}

/// Arrange to send a particular message to a channel if the task built by
/// this `TaskBuilder` fails.
pub fn spawn_named_with_send_on_failure<T: Send>(name: &'static str,
                                                 f: proc(): Send,
                                                 msg: T,
                                                 dest: Sender<T>,
                                                 native: bool) {
    let future_result = if native {
        TaskBuilder::new().named(name).native().try_future(proc() {
            f();
            error!("{} is dead", name);
        })
    } else {
        TaskBuilder::new().named(name).try_future(proc() {
            f();
            error!("{} is dead", name);
        })
    };

    let watched_name = name.to_string();
    let watcher_name = format!("{:s}Watcher", watched_name);
    TaskBuilder::new().named(watcher_name).spawn(proc() {
        match future_result.unwrap() {
            Ok(()) => (),
            Err(..) => {
                debug!("{:s} failed, notifying constellation", name);
                dest.send(msg);
            }
        }
    });
}
