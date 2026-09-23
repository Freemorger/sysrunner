// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 [Freemorger]

use std::collections::HashMap;

use crate::service::Service;

#[derive(Debug, Default)]
pub struct DepGraph {
    pub direct:  HashMap<String, Vec<String>>, // service A depends on: .. 
    pub reverse: HashMap<String, Vec<String>>, // service A is required for: .. 
}

impl DepGraph {
    pub fn from_services(services: &HashMap<String, Service>) -> Self {
        let mut direct:  HashMap<String, Vec<String>> = HashMap::new();
        let mut reverse: HashMap<String, Vec<String>> = HashMap::new();

        for (pnm, p) in services {
            let mut record_d: Vec<String> = Vec::new();

            for dep in &p.data.depends {
                record_d.push(dep.to_owned());

                if reverse.get(dep).is_some() {
                    let r = reverse.get_mut(dep).unwrap();
                    r.push(pnm.to_owned());
                } else {
                    let r: Vec<String> = vec![pnm.to_owned()];
                    reverse.insert(dep.to_owned(), r);
                }
            }

            direct.insert(pnm.clone(), record_d);
        }

        Self { direct, reverse }
    }
}
