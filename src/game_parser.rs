use std::io::Result;

use serde_json::{Map, Value};

use super::{
    frame::{self, Frame, PortData},
    game::{self, Game, NUM_PORTS},
    metadata,
    parse::{self, Indexed},
};

#[derive(Debug, Default)]
pub struct FrameEvents {
    pub pre: [Vec<frame::Pre>; NUM_PORTS],
    pub post: [Vec<frame::Post>; NUM_PORTS],
}

#[derive(Debug, Default)]
pub struct GameParser {
    pub start: Option<game::Start>,
    pub end: Option<game::End>,
    pub frames_start: Vec<frame::Start>,
    pub frames_end: Vec<frame::End>,
    pub frames_leaders: FrameEvents,
    pub frames_followers: FrameEvents,
    pub items: Vec<Vec<frame::Item>>,
    pub metadata: Option<Map<String, Value>>,
}

impl GameParser {
    pub fn into_game(self) -> Result<Game> {
        let mut num_players = 0;
        if let None = self.start {
            return Err(err!("missing start event"));
        } else if let Some(ref start) = self.start {
            num_players = start.players.len();
        }
        let start = self.start.ok_or_else(|| err!("missing start event"))?;
        let end = self.end.ok_or_else(|| err!("missing end event"))?;
        let ports: Vec<_> = start.players.iter().map(|p| p.port as usize).collect();

        let metadata_raw = self.metadata.unwrap_or_default();
        let metadata = metadata::parse(&metadata_raw)?;
        if let Some(ref players) = metadata.players {
            let meta_ports: Vec<_> = players.iter().map(|p| p.port as usize).collect();
            if meta_ports != ports {
                Err(err!(
                    "game-start ports ({:?}) != metadata ports ({:?})",
                    ports,
                    meta_ports
                ))?;
            }
        }

        let frame_count = self.frames_leaders.pre[ports[0]].len();

        for p in &ports {
            match self.frames_leaders.pre[*p].len() {
                n if n == frame_count => (),
                n => Err(err!("mismatched pre-frame counts: {}, {}", frame_count, n))?,
            }
        }

        for p in &ports {
            match self.frames_leaders.post[*p].len() {
                n if n == frame_count => (),
                n => Err(err!("mismatched post-frame counts: {}, {}", frame_count, n))?,
            }
        }

        let mut frames = Vec::with_capacity(frame_count);
        for n in 0..frame_count {
            frames.push(Frame {
                start: self.frames_start.get(n).copied(),
                end: self.frames_end.get(n).copied(),

                port1:
                    // TODO: extract this out to a function
                    if num_players >= 1 {
                        Some(PortData {
                            leader: frame::Data {
                                pre: self.frames_leaders.pre[ports[0]][n],
                                post: self.frames_leaders.post[ports[0]][n],
                            },
                            follower: {
                                let pre = &self.frames_followers.pre[ports[0]];
                                let post = &self.frames_followers.post[ports[0]];
                                match (pre.is_empty(), post.is_empty()) {
                                    (true, true) => None,
                                    (false, false) => Some(Box::new(frame::Data {
                                        pre: pre[n],
                                        post: post[n],
                                    })),
                                    _ => Err(err!("inconsistent follower data (frame: {})", n))?,
                                }
                            }
                        })
                    } else {
                        None
                    },
                    port2:
                        if num_players >= 2 {
                            Some(PortData {
                                leader: frame::Data {
                                    pre: self.frames_leaders.pre[ports[1]][n],
                                    post: self.frames_leaders.post[ports[1]][n],
                                },
                                follower: {
                                    let pre = &self.frames_followers.pre[ports[1]];
                                    let post = &self.frames_followers.post[ports[1]];
                                    match (pre.is_empty(), post.is_empty()) {
                                        (true, true) => None,
                                        (false, false) => Some(Box::new(frame::Data {
                                            pre: pre[n],
                                            post: post[n],
                                        })),
                                        _ => Err(err!("inconsistent follower data (frame: {})", n))?,
                                    }
                                }
                            })
                        } else {
                            None
                        },
                        port3:
                            if num_players >= 3 {
                                Some(PortData {
                                    leader: frame::Data {
                                        pre: self.frames_leaders.pre[ports[2]][n],
                                        post: self.frames_leaders.post[ports[2]][n],
                                    },
                                    follower: {
                                        let pre = &self.frames_followers.pre[ports[2]];
                                        let post = &self.frames_followers.post[ports[2]];
                                        match (pre.is_empty(), post.is_empty()) {
                                            (true, true) => None,
                                            (false, false) => Some(Box::new(frame::Data {
                                                pre: pre[n],
                                                post: post[n],
                                            })),
                                            _ => Err(err!("inconsistent follower data (frame: {})", n))?,
                                        }
                                    }
                                })
                            } else {
                                None
                            },
                            port4:
                                if num_players >= 4 {
                                    Some(PortData {
                                        leader: frame::Data {
                                            pre: self.frames_leaders.pre[ports[3]][n],
                                            post: self.frames_leaders.post[ports[3]][n],
                                        },
                                        follower: {
                                            let pre = &self.frames_followers.pre[ports[3]];
                                            let post = &self.frames_followers.post[ports[3]];
                                            match (pre.is_empty(), post.is_empty()) {
                                                (true, true) => None,
                                                (false, false) => Some(Box::new(frame::Data {
                                                    pre: pre[n],
                                                    post: post[n],
                                                })),
                                                _ => Err(err!("inconsistent follower data (frame: {})", n))?,
                                            }
                                        }
                                    })
                                } else {
                                    None
                                },
                                items: self.items.get(n).cloned(),
            });
        }

        Ok(Game {
            start: start,
            end: end,
            frames: frames,
            metadata: metadata,
            metadata_raw: metadata_raw,
        })
    }
}

fn append_frame_event<Id, Event>(
    v: &mut Vec<Event>,
    evt: parse::FrameEvent<Id, Event>,
) -> Result<()>
where
    Id: parse::Indexed,
    Event: Copy,
{
    let idx = evt.id.array_index();
    if idx == v.len() {
        v.push(evt.event);
    } else if idx < v.len() {
        // rollback
        v[idx] = evt.event;
    } else {
        // is this appropriate for Frame Start & Frame End?
        if let Some(&last) = v.last() {
            while v.len() < idx {
                v.push(last);
            }
        } else {
            Err(err!("missing initial frame data: {:?}", evt.id.index()))?;
        }
    }
    Ok(())
}

/// fills in missing frame data for eliminated players by duplicating their last-seen data
macro_rules! append_missing_frame_data {
    ( $arr: expr, $count: expr ) => {
        for f in $arr.iter_mut() {
            if let Some(&last) = f.last() {
                while f.len() < $count {
                    f.push(last);
                }
            }
        }
    };
}

impl parse::Handlers for GameParser {
    fn game_start(&mut self, s: game::Start) -> Result<()> {
        self.start = Some(s);
        Ok(())
    }

    fn game_end(&mut self, s: game::End) -> Result<()> {
        self.end = Some(s);
        Ok(())
    }

    fn frame_start(&mut self, evt: parse::FrameEvent<parse::FrameId, frame::Start>) -> Result<()> {
        append_frame_event(&mut self.frames_start, evt)?;
        Ok(())
    }

    fn frame_pre(&mut self, evt: parse::FrameEvent<parse::PortId, frame::Pre>) -> Result<()> {
        match evt.id.is_follower {
            true => Ok(append_frame_event(
                &mut self.frames_followers.pre[evt.id.port as usize],
                evt,
            )?),
            _ => Ok(append_frame_event(
                &mut self.frames_leaders.pre[evt.id.port as usize],
                evt,
            )?),
        }
    }

    fn frame_post(&mut self, evt: parse::FrameEvent<parse::PortId, frame::Post>) -> Result<()> {
        match evt.id.is_follower {
            true => Ok(append_frame_event(
                &mut self.frames_followers.post[evt.id.port as usize],
                evt,
            )?),
            _ => Ok(append_frame_event(
                &mut self.frames_leaders.post[evt.id.port as usize],
                evt,
            )?),
        }
    }

    fn frame_end(&mut self, evt: parse::FrameEvent<parse::FrameId, frame::End>) -> Result<()> {
        append_frame_event(&mut self.frames_end, evt)?;
        Ok(())
    }

    fn item(&mut self, evt: parse::FrameEvent<parse::FrameId, frame::Item>) -> Result<()> {
        let idx = evt.id.array_index();
        while self.items.len() <= idx {
            self.items.push(Vec::new());
        }
        let v = &mut self.items[idx];
        match v.iter().position(|i| i.id == evt.event.id) {
            Some(idx) => v[idx] = evt.event, // rollback
            _ => v.push(evt.event),
        };
        Ok(())
    }

    fn metadata(&mut self, metadata: Map<String, Value>) -> Result<()> {
        self.metadata = Some(metadata);
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        let frame_count = self
            .frames_leaders
            .pre
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0);

        append_missing_frame_data!(self.frames_leaders.pre, frame_count);
        append_missing_frame_data!(self.frames_leaders.post, frame_count);
        append_missing_frame_data!(self.frames_followers.pre, frame_count);
        append_missing_frame_data!(self.frames_followers.post, frame_count);

        while self.items.len() < frame_count {
            self.items.push(Vec::new());
        }

        Ok(())
    }
}
