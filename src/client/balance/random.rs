// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Random load balance implemention

use super::LoadBalance;
use crate::client::channel::RpcChannel;
use crate::client::discover::RpcChange;
use crate::server::Serve;
use core::cell::OnceCell;
use rand::Rng;
use std::fmt::Debug;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

/// A channel picker
#[derive(Debug)]
pub struct ChannelPicker<S>
where
    S: Serve,
    S::Req: Debug,
    S::Resp: Debug,
{
    shared_channels: Arc<WeightedChannels<S>>,
    sum_of_weights: isize,
    owned_channels: OnceCell<Vec<RpcChannel<S>>>,
    last_pick: Option<(usize, RpcChannel<S>)>,
}

impl<S> Iterator for ChannelPicker<S>
where
    S: Serve,
    S::Req: Debug,
    S::Resp: Debug,
{
    type Item = RpcChannel<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let shared_channels = &self.shared_channels.channels;
        if shared_channels.is_empty() {
            return None;
        }

        match &mut self.last_pick {
            None => {
                let (offset, channel) = pick_one(self.sum_of_weights, shared_channels)?;
                self.last_pick = Some((offset, channel.clone()));
                Some(channel.clone())
            },
            Some((last_offset, last_pick)) => {
                self.owned_channels.get_or_init(|| shared_channels.to_vec());
                let owned = self.owned_channels.get_mut().unwrap();

                self.sum_of_weights -= last_pick.config().instance.weight as isize;
                owned.remove(*last_offset);

                (*last_offset, *last_pick) = pick_one(self.sum_of_weights, owned)?;

                Some(last_pick.clone())
            },
        }
    }
}

/// Random load balance implemention
pub struct RandomBalance<S: Serve> {
    channels: AtomicPtr<Arc<WeightedChannels<S>>>,
}

impl<S: Serve> Drop for RandomBalance<S> {
    fn drop(&mut self) {
        let p = self.channels.load(Ordering::Acquire);
        if !p.is_null() {
            drop(unsafe { Box::from_raw(p) });
        }
    }
}

impl<S> RandomBalance<S>
where
    S: Serve + 'static,
    S::Req: Debug + Send,
    S::Resp: Debug + Send,
{
    /// Returns a empty [`RandomBalance`]
    pub fn new() -> Self {
        Self {
            channels: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

impl<S> LoadBalance<S> for RandomBalance<S>
where
    S: Serve + 'static,
    S::Req: Debug + Send,
    S::Resp: Debug + Send,
{
    type ChannelIter = ChannelPicker<S>;
    fn start_balance(&self, channels: Vec<RpcChannel<S>>) {
        self.channels.store(Box::into_raw(Box::new(Arc::new(WeightedChannels::from(channels)))), Ordering::Release)
    }
    fn get_picker(&self) -> Self::ChannelIter {
        let channels = unsafe { &*self.channels.load(Ordering::Acquire) };
        ChannelPicker {
            owned_channels: OnceCell::new(),
            last_pick: None,
            sum_of_weights: channels.sum_of_weights,
            shared_channels: channels.clone(),
        }
    }
    fn rebalance(&self, changes: Option<RpcChange<RpcChannel<S>>>) {
        let new_ptr = if let Some(changes) = changes {
            Box::into_raw(Box::new(Arc::new(WeightedChannels::from(changes.all))))
        } else {
            ptr::null_mut()
        };
        let p = self.channels.swap(new_ptr, Ordering::AcqRel);
        if !p.is_null() {
            drop(unsafe { Box::from_raw(p) });
        }
    }
}

#[inline]
fn pick_one<S: Serve>(weight: isize, iter: &[RpcChannel<S>]) -> Option<(usize, RpcChannel<S>)> {
    if weight == 0 {
        return None;
    }
    let mut weight = rand::thread_rng().gen_range(0..weight);
    for (offset, channel) in iter.iter().enumerate() {
        weight -= channel.config().instance.weight as isize;
        if weight <= 0 {
            return Some((offset, channel.clone()));
        }
    }
    None
}

#[derive(Clone, Default)]
struct WeightedChannels<S: Serve> {
    sum_of_weights: isize,
    channels: Vec<RpcChannel<S>>,
}

impl<S> Debug for WeightedChannels<S>
where
    S: Serve,
    S::Req: Debug,
    S::Resp: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeightedChannels")
            .field("sum_of_weights", &self.sum_of_weights)
            .field("channels", &self.channels)
            .finish()
    }
}

impl<S: Serve> From<Vec<RpcChannel<S>>> for WeightedChannels<S> {
    fn from(channels: Vec<RpcChannel<S>>) -> Self {
        let sum_of_weights = channels.iter().fold(0, |lhs, rhs| lhs + rhs.config().instance.weight as isize);
        Self { channels: channels, sum_of_weights }
    }
}
