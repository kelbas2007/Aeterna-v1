//! Bounded, authored sensory front end. It never receives labels or hidden
//! environment state. Its working object tracks travel in the joint checkpoint.
use super::vm::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub const SENSOR_WIDTH: usize = 32;
const FEATURES: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SensorFrame {
    Image {
        namespace: u16,
        pixels: Option<Vec<Vec<u8>>>,
    },
    Signals {
        namespace: u16,
        channels: Vec<Option<Vec<f64>>>,
    },
    Scene {
        namespace: u16,
        pixels: Vec<Vec<u16>>,
    },
    /// Simultaneous observations of one external event. Only leaf frames,
    /// at most one per physical modality; no recursive bundles or task labels.
    Multimodal {
        namespace: u16,
        frames: Vec<SensorFrame>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorEncoder {
    namespace: Option<u16>,
    tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Track {
    token: u16,
    x: i64,
    y: i64,
    area: i64,
    age: u8,
}

fn dimensions<T>(pixels: &[Vec<T>]) -> Result<(usize, usize)> {
    let h = pixels.len();
    let w = pixels.first().map_or(0, Vec::len);
    if h == 0 || w == 0 || h > 256 || w > 256 || pixels.iter().any(|r| r.len() != w) {
        return Err(Error::InvalidData);
    }
    Ok((h, w))
}

impl SensorEncoder {
    pub fn validate(&self) -> Result<()> {
        if self.tracks.len() > 6
            || self.tracks.iter().any(|t| {
                !(0..=1024).contains(&t.x)
                    || !(0..=1024).contains(&t.y)
                    || !(1..=1024).contains(&t.area)
                    || t.age > 3
            })
        {
            return Err(Error::InvalidData);
        }
        Ok(())
    }
    pub fn reset(&mut self) {
        self.namespace = None;
        self.tracks.clear();
    }

    /// Validate the complete input before changing working state.
    pub fn observe(&mut self, frame: &SensorFrame) -> Result<Vec<i64>> {
        let mut encoded = vec![0; SENSOR_WIDTH];
        match frame {
            SensorFrame::Image { namespace, pixels } => {
                encoded[0] = i64::from(*namespace) * 4;
                if let Some(pixels) = pixels {
                    let (h, w) = dimensions(pixels)?;
                    if h < 5 || w < 6 {
                        return Err(Error::InvalidData);
                    }
                    for by in 0..5 {
                        for bx in 0..6 {
                            let mut sum = 0u64;
                            let mut n = 0u64;
                            for row in pixels.iter().take((by + 1) * h / 5).skip(by * h / 5) {
                                for &v in row.iter().take((bx + 1) * w / 6).skip(bx * w / 6) {
                                    sum += u64::from(v);
                                    n += 1;
                                }
                            }
                            encoded[2 + by * 6 + bx] = (sum * 1024 / (255 * n)) as i64;
                        }
                    }
                    encoded[1] = (1_i64 << FEATURES) - 1;
                }
            }
            SensorFrame::Signals {
                namespace,
                channels,
            } => {
                if channels.len() != 9
                    || channels.iter().flatten().any(|xs| {
                        xs.is_empty()
                            || xs.len() > 2048
                            || xs.iter().any(|x| !x.is_finite() || x.abs() > 1e6)
                    })
                {
                    return Err(Error::InvalidData);
                }
                encoded[0] = i64::from(*namespace) * 4 + 1;
                for (channel, values) in channels.iter().enumerate() {
                    if let Some(xs) = values {
                        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
                        let sd = (xs.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                            / xs.len() as f64)
                            .sqrt();
                        let rough = xs.windows(2).map(|x| (x[1] - x[0]).abs()).sum::<f64>()
                            / xs.len().saturating_sub(1).max(1) as f64;
                        for (j, value) in [mean, sd, rough].iter().enumerate() {
                            let axis = channel * 3 + j;
                            encoded[2 + axis] = (value * 1024.0).round() as i64;
                            encoded[1] |= 1 << axis;
                        }
                    }
                }
            }
            SensorFrame::Scene { namespace, pixels } => {
                let (h, w) = dimensions(pixels)?;
                let mut counts = BTreeMap::<u16, usize>::new();
                for &v in pixels.iter().flatten() {
                    *counts.entry(v).or_default() += 1;
                }
                let background = *counts.iter().max_by_key(|(v, n)| (**n, **v)).unwrap().0;
                let mut seen = vec![false; h * w];
                let mut components = Vec::new();
                for y in 0..h {
                    for x in 0..w {
                        if seen[y * w + x] || pixels[y][x] == background {
                            continue;
                        }
                        let token = pixels[y][x];
                        let mut queue = VecDeque::from([(y, x)]);
                        seen[y * w + x] = true;
                        let (mut n, mut sx, mut sy) = (0, 0, 0);
                        while let Some((cy, cx)) = queue.pop_front() {
                            n += 1;
                            sx += cx;
                            sy += cy;
                            for (ny, nx) in [
                                (cy.wrapping_sub(1), cx),
                                (cy + 1, cx),
                                (cy, cx.wrapping_sub(1)),
                                (cy, cx + 1),
                            ] {
                                if ny < h && nx < w && !seen[ny * w + nx] && pixels[ny][nx] == token
                                {
                                    seen[ny * w + nx] = true;
                                    queue.push_back((ny, nx));
                                }
                            }
                        }
                        components.push(Track {
                            token,
                            x: (sx * 1024 / (n * w)) as i64,
                            y: (sy * 1024 / (n * h)) as i64,
                            area: (n * 1024 / (w * h)).max(1) as i64,
                            age: 0,
                        });
                    }
                }
                components.sort_by_key(|t| (std::cmp::Reverse(t.area), t.token, t.y, t.x));
                components.truncate(24);
                let mut tracks = if self.namespace == Some(*namespace) {
                    self.tracks.clone()
                } else {
                    Vec::new()
                };
                let mut movement = (0, 0);
                let mut matches = 0;
                // Appearance plus local displacement only. An occluded track
                // survives three frames; no hidden identity or global pose.
                for old in &mut tracks {
                    if let Some((i, _)) = components
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.token == old.token)
                        .map(|(i, c)| (i, (c.x - old.x).abs() + (c.y - old.y).abs()))
                        .filter(|(_, d)| *d <= 640)
                        .min_by_key(|(i, d)| (*d, *i))
                    {
                        let new = components.remove(i);
                        movement.0 += new.x - old.x;
                        movement.1 += new.y - old.y;
                        matches += 1;
                        *old = new;
                    } else {
                        old.age = old.age.saturating_add(1);
                    }
                }
                tracks.retain(|t| t.age <= 3);
                tracks.extend(
                    components
                        .into_iter()
                        .take(6usize.saturating_sub(tracks.len())),
                );
                encoded[0] = i64::from(*namespace) * 4 + 2;
                encoded[1] = (1_i64 << FEATURES) - 1;
                for (i, t) in tracks.iter().enumerate() {
                    encoded[2 + 4 * i..6 + 4 * i].copy_from_slice(&[
                        i64::from(t.token),
                        t.x,
                        t.y,
                        if t.age == 0 { t.area } else { -t.area },
                    ]);
                }
                encoded[26..].copy_from_slice(&[
                    h as i64,
                    w as i64,
                    counts.len() as i64,
                    tracks.len() as i64,
                    movement.0 / matches.max(1),
                    movement.1 / matches.max(1),
                ]);
                self.namespace = Some(*namespace);
                self.tracks = tracks;
            }
            SensorFrame::Multimodal { namespace, frames } => {
                if frames.is_empty()||frames.len()>3{return Err(Error::InvalidData)}
                let mut kinds=std::collections::BTreeSet::new();
                let mut next=self.clone();
                encoded[0]=i64::from(*namespace)*4+3;
                for child in frames {
                    let (k,ns)=match child {
                        SensorFrame::Image{namespace,..}=>(0,*namespace),
                        SensorFrame::Signals{namespace,..}=>(1,*namespace),
                        SensorFrame::Scene{namespace,..}=>(2,*namespace),
                        SensorFrame::Multimodal{..}=>return Err(Error::InvalidData),
                    };
                    if ns!=*namespace||!kinds.insert(k){return Err(Error::InvalidData)}
                    let values=next.observe(child)?;
                    // Compatibility projection only. The learning organism
                    // receives every raw child, not these thirty summaries.
                    for j in 0..10 {
                        let mut sum=0;let mut n=0;
                        for axis in j*3..j*3+3{if values[1]&(1_i64<<axis)!=0{sum+=values[2+axis];n+=1;}}
                        if n>0{let axis=k*10+j;encoded[2+axis]=sum/n;encoded[1]|=1_i64<<axis;}
                    }
                }
                *self=next;
            }
        }
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_scene_geometry_and_tracks_survive_a_checkpoint() {
        let mut e = SensorEncoder::default();
        let frame = |x| {
            let mut p = vec![vec![0; 8]; 8];
            p[3][x] = 7;
            SensorFrame::Scene {
                namespace: 3,
                pixels: p,
            }
        };
        let a = e.observe(&frame(2)).unwrap();
        let mut restored: SensorEncoder =
            serde_json::from_slice(&serde_json::to_vec(&e).unwrap()).unwrap();
        let b = restored.observe(&frame(3)).unwrap();
        assert_eq!(a[2], b[2]);
        assert!(b[3] > a[3]);
        assert!(b[30] > 0);
        assert_ne!(a, b);
        restored.validate().unwrap();
    }
    #[test]
    fn measurements_expose_only_acquired_channels_and_reject_nan() {
        let mut e = SensorEncoder::default();
        let mut c = vec![None; 9];
        c[4] = Some(vec![1.0, 2.0, 3.0]);
        let x = e
            .observe(&SensorFrame::Signals {
                namespace: 2,
                channels: c.clone(),
            })
            .unwrap();
        assert_eq!(x[1], 7 << 12);
        assert_eq!(x[14], 2048);
        assert!(x[2..14].iter().all(|v| *v == 0));
        c[0] = Some(vec![f64::NAN]);
        assert!(e
            .observe(&SensorFrame::Signals {
                namespace: 2,
                channels: c
            })
            .is_err());
    }
}
