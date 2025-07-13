use crate::registries::Item;

use std::fmt::{Display, Formatter, Result};

#[derive(Clone)]
pub struct Slot {
    pub item: Item,
    pub count: u32,
}

pub struct Inventory {
    pub owner_money: Option<u32>,
    pub slots: Vec<Slot>,
    pub max_slots: usize,
}

impl Inventory {
    pub fn new(owner_money: Option<u32>) -> Self {
        Self {
            owner_money,
            slots: Vec::new(),
            max_slots: 32,
        }
    }

    pub fn add_item(&mut self, item: Item, mut quantity: u32) -> bool {
        // Bestehende Stacks auffüllen
        for slot in self.slots.iter_mut() {
            if slot.item == item && slot.count < item.stack_size {
                let space = item.stack_size - slot.count;
                let add = quantity.min(space);
                slot.count += add;
                quantity -= add;
                if quantity == 0 {
                    return true;
                }
            }
        }

        // Neue Stacks anlegen, wenn Platz ist
        while quantity > 0 {
            if self.slots.len() < self.max_slots {
                let add = quantity.min(item.stack_size);
                self.slots.push(Slot {
                    item: item.clone(),
                    count: add,
                });
                quantity -= add;
            } else {
                eprintln!("⚠ No free inventory space for {}!", item.name);
                return false;
            }
        }
        true
    }

    pub fn remove_item(&mut self, item: &Item, mut quantity: u32) -> bool {
        let mut removed = 0;

        for slot in self.slots.iter_mut() {
            if &slot.item == item {
                let can_remove = (quantity - removed).min(slot.count);
                slot.count -= can_remove;
                removed += can_remove;
            }
        }

        self.slots.retain(|s| s.count > 0);

        if removed < quantity {
            eprintln!("⚠ Not enough {} to remove!", item.name);
            return false;
        }
        true
    }

    pub fn total_items_of(&self, item: &Item) -> u32 {
        self.slots
            .iter()
            .filter(|s| &s.item == item)
            .map(|s| s.count)
            .sum()
    }

    pub fn total_items(&self) -> u32 {
        self.slots.iter().map(|s| s.count).sum()
    }

    pub fn has_item(&self, item: &Item, quantity: u32) -> bool {
        self.total_items_of(item) >= quantity
    }
}

impl Display for Inventory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut slots = self.slots.clone();
        slots.sort_by(|a, b| a.item.name.to_lowercase().cmp(&b.item.name.to_lowercase()));

        let slot_count = slots.len();
        let columns = match slot_count {
            0..=8 => 1,
            9..=26 => 2,
            _ => 3,
        };

        let c_width = 21;
        let a_width = 6;
        let ft_width = 13;

        let ctl = "╭";
        let ctr = "╮";
        let cbl = "╰";
        let cbr = "╯";
        let st = "┬";
        let sl = "├";
        let sr = "┤";
        let sb = "┴";
        let sm = "┼";
        let lv = "│";
        let lh = "─";

        let mut output = String::new();

        let b_row = |left: &str, mid: &str, right: &str| {
            let parts = (0..columns)
                .map(|_| format!("{0}{1}{0}", lh.repeat(c_width + 2), mid))
                .collect::<Vec<_>>()
                .join(&lh.repeat(a_width + 2));
            format!("{left}{parts}{right}\n")
        };

        let h_row = || {
            let header = (0..columns)
                .map(|_| format!(" {:<c_width$} {lv} {:>a_width$} {lv}", "Item", "Amount"))
                .collect::<Vec<_>>()
                .join(lv);
            format!("{lv}{header}\n")
        };

        let c_row = |items: &[(String, u32)]| {
            let mut row = String::new();
            for (name, amount) in items.iter() {
                row += &format!(" {:<c_width$} {lv} {:>a_width$}x {lv}", name, amount);
            }
            for _ in 0..(columns - items.len()) {
                row += &format!(" {:<c_width$} {lv} {:>a_width$} {lv}", "", "");
            }
            format!("{lv}{row}\n")
        };

        output += &b_row(ctl, st, ctr);
        output += &h_row();
        output += &b_row(sl, sm, sr);

        if slot_count == 0 {
            output += &c_row(&[]);
        } else {
            for chunk in slots.chunks(columns) {
                let group = chunk
                    .iter()
                    .map(|s| (s.item.name.clone(), s.count))
                    .collect::<Vec<_>>();
                output += &c_row(&group);
            }
        }

        let t_width = (c_width + a_width + 6) * columns + 1;

        if columns == 1 {
            output += &format!(
                "{sl}{lh:ft_width$}{st}{lh:(c_width + 2 - (ft_width + 1))$}{sb}{lh:(a_width + 2)$}{sr}\n"
            );
        } else if columns == 2 {
            output += &format!(
                "{sl}{lh:ft_width$}{st}{lh:(c_width + 2 - (ft_width + 1))$}{sb}{lh:(a_width + 2)$}{sb}{lh:(c_width + 2)$}{sb}{lh:(a_width + 2)$}{sr}\n"
            );
        } else {
            output += &format!("{sl}{lh:ft_width$}{st}{lh:(c_width + 2 - (ft_width + 1))$}{sb}{lh:(a_width + 2)$}{sb}");
            for _ in 0..(columns - 2) {
                output += &format!("{lh:(c_width + 2)$}{sb}{lh:(a_width + 2)$}{sb}");
            }
            output += &format!("{lh:(c_width + 2)$}{sb}{lh:(a_width + 2)$}{sr}\n");
        }

        output += &format!("{lv} Total Items │ {:>width$} {lv}\n", format!("{}/{}", self.total_items(), self.max_slots as u32 * 64), width = t_width - 18);
        output += &format!("{lv} Stacks      │ {:>width$} {lv}\n", format!("{}/{}", self.slots.len(), self.max_slots), width = t_width - 18);
        output += &format!("{lv} Money       │ {:>width$} {lv}\n", self.owner_money.map_or("N/A".into(), |v| v.to_string()), width = t_width - 19);
        output += &format!("{cbl}{lh:ft_width$}{sb}{lh:(t_width - 16)$}{cbr}\n");

        write!(f, "{output}")
    }
}
