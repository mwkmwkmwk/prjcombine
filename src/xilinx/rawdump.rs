use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::path::Path;
use crate::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct WireIdx {
    idx: u32,
}

impl WireIdx {
    pub const NONE: WireIdx = WireIdx { idx: u32::MAX };
    fn from_raw(i: usize) -> WireIdx {
        assert!(i < u32::MAX as usize);
        WireIdx {idx: i as u32}
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct SpeedIdx {
    idx: u32,
}

impl SpeedIdx {
    pub const NONE: SpeedIdx = SpeedIdx { idx: u32::MAX };
    pub const UNKNOWN: SpeedIdx = SpeedIdx { idx: u32::MAX - 1 };
    fn from_raw(i: usize) -> SpeedIdx {
        assert!(i < (u32::MAX - 1) as usize);
        SpeedIdx {idx: i as u32}
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct NodeIdx {
    idx: u32,
}

impl NodeIdx {
    pub const NONE: NodeIdx = NodeIdx { idx: u32::MAX };
    pub const PENDING: NodeIdx = NodeIdx { idx: u32::MAX - 1 };
    fn from_raw(i: usize) -> NodeIdx {
        assert!(i < (u32::MAX - 1) as usize);
        NodeIdx {idx: i as u32}
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Source {
    ISE,
    Vivado,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkSiteSlot {
    Single(u16),
    Indexed(u16, u8),
    Xy(u16, u8, u8),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkSitePinDir {
    Input,
    Output,
    Bidir,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkSitePin {
    pub dir: TkSitePinDir,
    pub wire: WireIdx,
    pub speed: SpeedIdx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TkSite {
    pub slot: TkSiteSlot,
    pub kind: String,
    pub pins: HashMap<String, TkSitePin>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkWire {
    Internal(SpeedIdx),
    Connected(usize),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkPipMode {
    Const(SpeedIdx),
    Variable(usize),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkPipInversion {
    Never,
    Always,
    Prog,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum TkPipDirection {
    Uni,
    BiFwd,
    BiBwd,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkPip {
    pub is_buf: bool,
    pub is_excluded: bool,
    pub is_test: bool,
    pub inversion: TkPipInversion,
    pub direction: TkPipDirection,
    pub mode: TkPipMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileKind {
    pub sites: Vec<TkSite>,
    pub sites_by_slot: HashMap<TkSiteSlot, usize>,
    pub wires: HashMap<WireIdx, TkWire>,
    pub conn_wires: Vec<WireIdx>,
    pub pips: HashMap<(WireIdx, WireIdx), TkPip>,
    pub var_pips: Vec<(WireIdx, WireIdx)>,
    pub tiles: Vec<Coord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub name: String,
    pub kind: String,
    pub sites: Vec<Option<String>>,
    #[serde(skip)]
    pub conn_wires: Vec<NodeIdx>,
    pub var_pips: Vec<SpeedIdx>,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkNode {
    pub base: Coord,
    pub template: u32,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct TkNodeTemplateWire {
    pub delta: Coord,
    pub wire: WireIdx,
    pub speed: SpeedIdx,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct TkNodeTemplate {
    pub wires: Vec<TkNodeTemplateWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartCombo {
    pub name: String,
    pub device: String,
    pub package: String,
    pub speed: String,
    pub temp: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct PkgPin {
    pub pad: Option<String>,
    pub pin: String,
    pub vref_bank: Option<u32>,
    pub vcco_bank: Option<u32>,
    pub func: String,
    pub tracelen_um: Option<u32>,
    pub delay_min_fs: Option<u32>,
    pub delay_max_fs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub part: String,
    pub family: String,
    pub source: Source,
    pub width: u16,
    pub height: u16,
    pub tile_kinds: HashMap<String, TileKind>,
    pub tiles: HashMap<Coord, Tile>,
    pub speeds: Vec<String>,
    pub nodes: Vec<TkNode>,
    pub templates: Vec<TkNodeTemplate>,
    pub wires: Vec<String>,
    pub slot_kinds: Vec<String>,
    pub packages: HashMap<String, Vec<PkgPin>>,
    pub combos: Vec<PartCombo>,
}

pub struct PartBuilder {
    pub part: Part,
    pub tiles_by_name: HashMap<String, Coord>,
    pub speeds_by_name: HashMap<String, SpeedIdx>,
    pub templates_idx: HashMap<TkNodeTemplate, u32>,
    pub wires_by_name: HashMap<String, WireIdx>,
    pub slot_kinds_by_name: HashMap<String, u16>,
}

fn split_xy(s: &str) -> Option<(&str, u32, u32)> {
    let (l, r) = match s.rfind("_X") {
        None => return None,
        Some(pos) => (&s[..pos], &s[pos+2..]),
    };
    let (x, y) = match r.rfind("Y") {
        None => return None,
        Some(pos) => (&r[..pos], &r[pos+1..]),
    };
    let x = match x.parse::<u32>() {
        Err(_) => return None,
        Ok(x) => x,
    };
    let y = match y.parse::<u32>() {
        Err(_) => return None,
        Ok(y) => y,
    };
    Some((l, x, y))
}

#[cfg(test)]
mod tests {
    #[test]
    fn split_xy_test() {
        assert_eq!(super::split_xy("SLICE_X123Y456"), Some(("SLICE", 123, 456)));
    }
}

fn get_lastnum(s: &str) -> u8 {
    let mut num : Option<u8> = None;
    for c in s.chars() {
        if c.is_ascii_digit() {
            let v = c.to_digit(10).unwrap() as u8;
            num = Some(match num {
                None => v,
                Some(c) => c * 10 + v,
            })
        } else {
            num = None
        }
    }
    num.unwrap()
}

impl PartBuilder {
    pub fn new(part: String, family: String, source: Source, width: u16, height: u16) -> Self {
        PartBuilder {
            part: Part {
                part,
                family,
                source,
                width,
                height,
                tile_kinds: HashMap::new(),
                tiles: HashMap::new(),
                speeds: Vec::new(),
                nodes: Vec::new(),
                templates: Vec::new(),
                wires: Vec::new(),
                slot_kinds: Vec::new(),
                packages: HashMap::new(),
                combos: Vec::new(),
            },
            tiles_by_name: HashMap::new(),
            speeds_by_name: HashMap::new(),
            templates_idx: HashMap::new(),
            wires_by_name: HashMap::new(),
            slot_kinds_by_name: HashMap::new(),
        }
    }

    fn slotify<'a>(&mut self, sites: &'a [(&'a str, &'a str, Vec<(&'a str, TkSitePinDir, Option<&'a str>, Option<&'a str>)>)]) -> HashMap<&'a str, TkSiteSlot> {
        fn from_pinnum(pins: &[(&str, TkSitePinDir, Option<&str>, Option<&str>)], pin: &str) -> u8 {
            for (n, _, w, _) in pins {
                if *n == pin {
                    return get_lastnum(w.unwrap());
                }
            }
            panic!("key pin {} not found", pin);
        }

        let mut res: HashMap<&'a str, TkSiteSlot> = HashMap::new();
        let mut minxy: HashMap<u16, (u32, u32)> = HashMap::new();
        for (n, _, _) in sites {
            if let Some((base, x, y)) = split_xy(n) {
                let base = self.slot_kind_to_idx(base);
                let e = minxy.entry(base).or_insert((x, y));
                if x < e.0 {
                    e.0 = x;
                }
                if y < e.1 {
                    e.1 = y;
                }
            }
        }
        let mut slots: HashSet<TkSiteSlot> = HashSet::new();
        for (n, k, p) in sites {
            let slot = if self.part.family == "xc4000e" || self.part.family == "xc4000ex" || self.part.family == "xc4000xla" || self.part.family == "xc4000xv" || self.part.family == "spartanxl" {
                if let Some(urpos) = n.find("_R") {
                    if let Some(dpos) = n.find(".") {
                        TkSiteSlot::Indexed(self.slot_kind_to_idx(&n[..urpos]), n[dpos+1..].parse::<u8>().unwrap())
                    } else {
                        TkSiteSlot::Single(self.slot_kind_to_idx(&n[..urpos]))
                    }
                } else if *k == "IOB" || *k == "CLKIOB" || *k == "FCLKIOB" {
                    TkSiteSlot::Indexed(self.slot_kind_to_idx("IOB"), from_pinnum(p, "O"))
                } else if *k == "CIN" || *k == "COUT" || *k == "BUFF" {
                    TkSiteSlot::Single(self.slot_kind_to_idx(k))
                } else if *k == "PRI-CLK" {
                    TkSiteSlot::Single(self.slot_kind_to_idx("BUFGP"))
                } else if *k == "SEC-CLK" {
                    TkSiteSlot::Single(self.slot_kind_to_idx("BUFGS"))
                } else if *k == "BUFG" || *k == "BUFGE" || *k == "BUFGLS" {
                    let pos = n.find("_").unwrap();
                    TkSiteSlot::Indexed(self.slot_kind_to_idx(&n[..pos]), match &n[pos..] {
                        "_WNW" => 0,
                        "_ENE" => 1,
                        "_NNE" => 2,
                        "_SSE" => 3,
                        "_ESE" => 4,
                        "_WSW" => 5,
                        "_SSW" => 6,
                        "_NNW" => 7,
                        _ => panic!("cannot match {}", n),
                    })
                } else {
                    TkSiteSlot::Single(self.slot_kind_to_idx(n))
                }
            } else if self.part.family == "virtex" || self.part.family == "virtexe" {
                match *k {
                    "IOB" | "EMPTYIOB" | "PCIIOB" | "DLLIOB" => TkSiteSlot::Indexed(self.slot_kind_to_idx("IOB"), from_pinnum(p, "I")),
                    "TBUF" => TkSiteSlot::Indexed(self.slot_kind_to_idx(k), from_pinnum(p, "O")),
                    "SLICE" => TkSiteSlot::Indexed(self.slot_kind_to_idx(k), from_pinnum(p, "CIN")),
                    "GCLKIOB" => TkSiteSlot::Indexed(self.slot_kind_to_idx(k), from_pinnum(p, "GCLKOUT")),
                    "GCLK" => TkSiteSlot::Indexed(self.slot_kind_to_idx(k), from_pinnum(p, "CE")),
                    "DLL" => TkSiteSlot::Indexed(self.slot_kind_to_idx(k), match *n {
                        "DLL0" => 0,
                        "DLL1" => 1,
                        "DLL2" => 2,
                        "DLL3" => 3,
                        "DLL0P" => 0,
                        "DLL1P" => 1,
                        "DLL2P" => 2,
                        "DLL3P" => 3,
                        "DLL0S" => 4,
                        "DLL1S" => 5,
                        "DLL2S" => 6,
                        "DLL3S" => 7,
                        _ => panic!("cannot match {}", n),
                    }),
                    _ => TkSiteSlot::Single(self.slot_kind_to_idx(k))
                }
            } else if *k == "TBUF" && self.part.family.starts_with("virtex2") {
                TkSiteSlot::Indexed(self.slot_kind_to_idx(k), from_pinnum(p, "O"))
            } else if (*k == "GTIPAD" || *k == "GTOPAD") && self.part.family == "virtex2p" {
                let idx : u8 = match n.as_bytes()[2] {
                    b'P' => 0,
                    b'N' => 1,
                    _ => panic!("weird GT pad"),
                };
                TkSiteSlot::Indexed(self.slot_kind_to_idx(k), idx)
            } else if let Some((base, x, y)) = split_xy(n) {
                let base = self.slot_kind_to_idx(base);
                let (bx, by) = *minxy.get(&base).unwrap();
                TkSiteSlot::Xy(base, (x - bx) as u8, (y - by) as u8)
            } else if (self.part.family.starts_with("virtex2") || self.part.family.starts_with("spartan3")) && (k.starts_with("IOB") || k.starts_with("IBUF") || k.starts_with("DIFF")) {
                TkSiteSlot::Indexed(self.slot_kind_to_idx("IOB"), from_pinnum(p, "T"))
            } else if ((self.part.family.starts_with("virtex2") || self.part.family == "spartan3") && k.starts_with("DCI")) || (self.part.family == "spartan3" && *k == "BUFGMUX") {
                TkSiteSlot::Indexed(self.slot_kind_to_idx(k), get_lastnum(n))
            } else if self.part.family.starts_with("virtex2") && *k == "BUFGMUX" {
                TkSiteSlot::Indexed(self.slot_kind_to_idx(k), n[7..8].parse::<u8>().unwrap())
            } else if self.part.family == "spartan6" && k.starts_with("IOB") {
                TkSiteSlot::Indexed(self.slot_kind_to_idx("IOB"), from_pinnum(p, "PADOUT"))
            } else {
                TkSiteSlot::Single(self.slot_kind_to_idx(n))
            };
            assert!(!slots.contains(&slot));
            slots.insert(slot);
            res.insert(n, slot);
        }
        res
    }

    pub fn add_tile(
        &mut self,
        coord: Coord,
        name: String,
        kind: String,
        sites: &[(&str, &str, Vec<(&str, TkSitePinDir, Option<&str>, Option<&str>)>)],
        wires: &[(&str, Option<&str>)],
        pips: &[(&str, &str, bool, bool, bool, TkPipInversion, TkPipDirection, Option<&str>)],
    ) {
        assert!(coord.x < self.part.width);
        assert!(coord.y < self.part.height);

        let wires : Vec<_> = wires.iter().map(|(n, s)| (
            self.wire_to_idx(n), self.speed_to_idx(*s)
        )).collect();
        let pips : Vec<_> = pips.iter().map(|(wf, wt, ib, ie, it, inv, dir, s)| (
            self.wire_to_idx(wf),
            self.wire_to_idx(wt),
            *ib,
            *ie,
            *it,
            *inv,
            *dir,
            self.speed_to_idx(*s),
        )).collect();
        let slots = self.slotify(sites);
        let sites_raw : Vec<_> = sites.iter().map(|(n, k, p)| (
            *slots.get(n).unwrap(),
            *n,
            *k,
            p.iter().map(|(n, d, w, s)| (
                *n,
                *d,
                match w {Some(w) => self.wire_to_idx(w), None => WireIdx::NONE},
                self.speed_to_idx(*s),
            )).collect::<Vec<_>>()
        )).collect();

        let mut sites: Vec<Option<String>> = Vec::new();
        let mut conn_wires: Vec<NodeIdx> = Vec::new();
        let mut var_pips: Vec<SpeedIdx> = Vec::new();

        let mut set_site = |i, s| {
            if sites.len() <= i {
                sites.resize(i + 1, None);
            }
            sites[i] = Some(s);
        };

        let mut set_conn_wire = |i, ni| {
            if conn_wires.len() <= i {
                conn_wires.resize(i + 1, NodeIdx::NONE);
            }
            conn_wires[i] = ni;
        };

        let mut set_var_pip = |i, si| {
            if var_pips.len() <= i {
                var_pips.resize(i + 1, SpeedIdx::NONE);
            }
            var_pips[i] = si;
        };

        match self.part.tile_kinds.get_mut(&kind) {
            Some(tk) => {
                for (slot, name, kind, pins) in sites_raw {
                    match tk.sites_by_slot.get(&slot) {
                        Some(idx) => {
                            let site = &mut tk.sites[*idx];
                            for (n, _, w, s) in pins {
                                let pin = site.pins.get_mut(n).unwrap();
                                if w == WireIdx::NONE { continue; }
                                if pin.wire != WireIdx::NONE && pin.wire != w {
                                    panic!("pin wire mismatch");
                                }
                                pin.wire = w;
                                if pin.speed != s {
                                    panic!("pin speed mismatch");
                                }
                            }
                            set_site(*idx, name.to_string());
                        },
                        None => {
                            let i = tk.sites.len();
                            tk.sites.push(TkSite {
                                slot: slot,
                                kind: kind.to_string(),
                                pins: pins.iter().map(|(n, d, w, s)| (n.to_string(), TkSitePin {dir: *d, wire: *w, speed: *s})).collect(),
                            });
                            tk.sites_by_slot.insert(slot, i);
                            set_site(i, name.to_string());
                        },
                    }
                }

                // Process wires.
                for (n, s) in wires {
                    match tk.wires.get(&n) {
                        None => {
                            let i = tk.conn_wires.len();
                            tk.wires.insert(n, TkWire::Connected(i));
                            tk.conn_wires.push(n);
                            set_conn_wire(i, NodeIdx::PENDING);
                        },
                        Some(TkWire::Internal(cs)) => {
                            if *cs != s {
                                let i = tk.conn_wires.len();
                                tk.wires.insert(n, TkWire::Connected(i));
                                tk.conn_wires.push(n);
                                set_conn_wire(i, NodeIdx::PENDING);
                                for crd in &tk.tiles {
                                    self.part.tiles.get_mut(&crd).unwrap().set_conn_wire(i, NodeIdx::PENDING);
                                }
                            }
                        },
                        Some(TkWire::Connected(i)) => {
                            set_conn_wire(*i, NodeIdx::PENDING);
                        },
                    }
                }

                // Process pips.
                for (wf, wt, ib, ie, it, inv, dir, s) in pips {
                    let k = (wf, wt);
                    match match tk.pips.get(&k) {
                        None => {
                            tk.pips.insert(k, TkPip {
                                is_buf: ib,
                                is_excluded: ie,
                                is_test: it,
                                inversion: inv,
                                direction: dir,
                                mode: TkPipMode::Const(s),
                            });
                            None
                        },
                        Some(TkPip{is_buf, is_excluded, is_test, inversion, direction, mode}) => {
                            if *is_buf != ib || *is_excluded != ie || *is_test != it || *inversion != inv || *direction != dir {
                                panic!("pip flags mismatch {} {} {} {} {} {} {} {:?} {} {}, {}, {:?}", name, kind, self.part.wires[wf.idx as usize], self.part.wires[wt.idx as usize], is_buf, is_excluded, is_test, inversion, ib, ie, it, inv);
                            }
                            match mode {
                                TkPipMode::Const(cs) => {
                                    if *cs != s {
                                        let i = tk.var_pips.len();
                                        tk.var_pips.push((wf, wt));
                                        set_var_pip(i, s);
                                        for crd in &tk.tiles {
                                            let tile = self.part.tiles.get_mut(&crd).unwrap();
                                            tile.set_var_pip(i, if tile.has_wire(tk, wf) && tile.has_wire(tk, wt) { *cs } else { SpeedIdx::NONE });
                                        }
                                        Some(i)
                                    } else {
                                        None
                                    }
                                },
                                TkPipMode::Variable(i) => {
                                    set_var_pip(*i, s);
                                    None
                                }
                            }
                        },
                    } {
                        None => (),
                        Some(i) => tk.pips.get_mut(&k).unwrap().mode = TkPipMode::Variable(i),
                    }
                }

                // Add the current tile.
                tk.tiles.push(coord);
            },
            None => {
                self.part.tile_kinds.insert(kind.to_string(), TileKind {
                    sites: sites_raw.iter().map(|(slot, _, kind, pins)| TkSite {
                        slot: *slot,
                        kind: kind.to_string(),
                        pins: pins.iter().map(|(n, d, w, s)| (n.to_string(), TkSitePin {dir: *d, wire: *w, speed: *s})).collect(),
                    }).collect(),
                    sites_by_slot: sites_raw.iter().enumerate().map(|(idx, (slot, _, _, _))| (*slot, idx)).collect(),
                    wires: wires.iter().map(|(n, s)| (
                        *n, TkWire::Internal(*s)
                    )).collect(),
                    conn_wires: Vec::new(),
                    pips: pips.iter().map(|(wf, wt, ib, ie, it, inv, dir, s)| (
                        (*wf, *wt), TkPip {
                            is_buf: *ib,
                            is_excluded: *ie,
                            is_test: *it,
                            inversion: *inv,
                            direction: *dir,
                            mode: TkPipMode::Const(*s),
                        }
                    )).collect(),
                    var_pips: Vec::new(),
                    tiles: vec![coord],
                });
                for (_, name, _, _) in sites_raw {
                    sites.push(Some(name.to_string()));
                }
            },
        }
        self.part.tiles.insert(coord, Tile {
            name: name.clone(),
            kind,
            sites,
            conn_wires,
            var_pips,
        });
        self.tiles_by_name.insert(name, coord);
    }

    pub fn add_node(&mut self, wires: &[(&str, &str, Option<&str>)]) {
        let wires: Vec<_> = wires.iter().map(|(t, w, s)| (
            *self.tiles_by_name.get(*t).unwrap(),
            self.wire_to_idx(w),
            self.speed_to_idx(*s),
        )).collect();
        if wires.len() == 1 {
            let (coord, wire, speed) = wires[0];
            let tile = self.part.tiles.get(&coord).unwrap();
            let tk = self.part.tile_kinds.get(&tile.kind).unwrap();
            if *tk.wires.get(&wire).unwrap() == TkWire::Internal(speed) {
                return;
            }
        }
        let bx = wires.iter().map(|(t, _, _)| t.x).min().unwrap();
        let by = wires.iter().map(|(t, _, _)| t.y).min().unwrap();
        let mut twires: Vec<_> = wires.iter().map(|(t, w, s)| TkNodeTemplateWire {
            delta: Coord{x: t.x - bx, y: t.y - by},
            wire: *w,
            speed: *s,
        }).collect();
        twires.sort();
        let template = TkNodeTemplate {
            wires: twires,
        };
        let tidx = match self.templates_idx.get(&template) {
            None => {
                let i = self.part.templates.len();
                if i > u32::MAX as usize {
                    panic!("out of templates");
                }
                let i = i as u32;
                self.part.templates.push(template.clone());
                self.templates_idx.insert(template, i);
                i
            },
            Some(i) => *i
        };
        let node = NodeIdx::from_raw(self.part.nodes.len());
        self.part.nodes.push(TkNode {
            base: Coord{x: bx, y: by},
            template: tidx,
        });
        for (coord, wire, _) in wires {
            let tile = self.part.tiles.get(&coord).unwrap();
            let tk = self.part.tile_kinds.get_mut(&tile.kind).unwrap();
            let w = tk.wires.get_mut(&wire).unwrap();
            let idx = match *w {
                TkWire::Internal(_) => {
                    let i = tk.conn_wires.len();
                    *w = TkWire::Connected(i);
                    tk.conn_wires.push(wire);
                    for crd in &tk.tiles {
                        self.part.tiles.get_mut(&crd).unwrap().set_conn_wire(i, NodeIdx::PENDING);
                    }
                    i
                },
                TkWire::Connected(i) => i,
            };
            self.part.tiles.get_mut(&coord).unwrap().set_conn_wire(idx, node);
        }
    }

    pub fn add_package(&mut self, name: String, pins: Vec<PkgPin>) {
        self.part.packages.insert(name, pins);
    }
    pub fn add_combo(&mut self, name: String, device: String, package: String, speed: String, temp: String) {
        self.part.combos.push(PartCombo {name, device, package, speed, temp});
    }

    pub fn wire_to_idx(&mut self, s: &str) -> WireIdx {
        match self.wires_by_name.get(s) {
            None => {
                let i = WireIdx::from_raw(self.part.wires.len());
                self.part.wires.push(s.to_string());
                self.wires_by_name.insert(s.to_string(), i);
                i
            },
            Some(i) => *i
        }
    }
    pub fn speed_to_idx(&mut self, s: Option<&str>) -> SpeedIdx {
        match s {
            None => SpeedIdx::UNKNOWN,
            Some(s) => match self.speeds_by_name.get(s) {
                None => {
                    let i = SpeedIdx::from_raw(self.part.speeds.len());
                    self.part.speeds.push(s.to_string());
                    self.speeds_by_name.insert(s.to_string(), i);
                    i
                },
                Some(i) => *i
            }
        }
    }

    pub fn slot_kind_to_idx(&mut self, s: &str) -> u16 {
        match self.slot_kinds_by_name.get(s) {
            None => {
                let i = self.part.slot_kinds.len();
                if i > u16::MAX as usize {
                    panic!("out of slot kinds");
                }
                let i = i as u16;
                self.part.slot_kinds.push(s.to_string());
                self.slot_kinds_by_name.insert(s.to_string(), i);
                i
            },
            Some(i) => *i
        }
    }

    pub fn finish(self) -> Part {
        self.part
    }
}

impl Part {
    pub fn print_wire(&self, w: WireIdx) -> &str {
        if w == WireIdx::NONE {
            "[NONE]"
        } else {
            &self.wires[w.idx as usize]
        }
    }

    pub fn print_speed(&self, s: SpeedIdx) -> &str {
        if s == SpeedIdx::NONE {
            "[NONE]"
        } else if s == SpeedIdx::UNKNOWN {
            "[UNKNOWN]"
        } else {
            &self.speeds[s.idx as usize]
        }
    }

    pub fn print_slot_kind(&self, sk: u16) -> &str {
        &self.slot_kinds[sk as usize]
    }

    pub fn post_deserialize(&mut self) {
        for (i, node) in self.nodes.iter().enumerate() {
            let template = &self.templates[node.template as usize];
            for w in template.wires.iter() {
                let coord = Coord {x: node.base.x + w.delta.x, y: node.base.y + w.delta.y};
                let tile = self.tiles.get_mut(&coord).unwrap();
                let tk = self.tile_kinds.get(&tile.kind).unwrap();
                let wire = tk.wires.get(&w.wire).unwrap();
                let idx = match wire {
                    TkWire::Internal(_) => panic!("node on internal wire"),
                    TkWire::Connected(idx) => *idx,
                };
                tile.set_conn_wire(idx, NodeIdx::from_raw(i));
            }
        }
    }

    pub fn from_file<P: AsRef<Path>> (path: P) -> Result<Self, Error> {
        let f = File::open(path)?;
        let xz = xz2::read::XzDecoder::new(f);
        let mut res: Part = bincode::deserialize_from(xz).unwrap();
        res.post_deserialize();
        Ok(res)
    }

    pub fn to_file<P: AsRef<Path>> (&self, path: P) -> Result<(), Error> {
        let f = File::create(path)?;
        let mut xz = xz2::write::XzEncoder::new(f, 9);
        bincode::serialize_into(&mut xz, self).unwrap();
        xz.finish()?;
        Ok(())
    }
}

impl Tile {
    pub fn set_conn_wire(&mut self, idx: usize, val: NodeIdx) {
        if self.conn_wires.len() <= idx {
            self.conn_wires.resize(idx + 1, NodeIdx::NONE);
        }
        if self.conn_wires[idx] != NodeIdx::PENDING && self.conn_wires[idx] != NodeIdx::NONE && val != NodeIdx::PENDING {
            panic!("conn wire double set {}", self.name);
        }
        self.conn_wires[idx] = val;
    }
    pub fn set_var_pip(&mut self, idx: usize, val: SpeedIdx) {
        if self.var_pips.len() <= idx {
            self.var_pips.resize(idx + 1, SpeedIdx::NONE);
        }
        self.var_pips[idx] = val;
    }
    pub fn has_wire(&self, tk: &TileKind, w: WireIdx) -> bool {
        match tk.wires.get(&w) {
            None => false,
            Some(TkWire::Internal(_)) => true,
            Some(TkWire::Connected(idx)) => {
                match self.conn_wires.get(*idx) {
                    None => false,
                    Some(ni) => *ni != NodeIdx::NONE,
                }
            }
        }
    }
}
