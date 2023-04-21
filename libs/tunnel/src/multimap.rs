use std::{collections::HashMap, hash::Hash};

// Custom very simple multimap
// it has get_main for getting things through the main key
// it has get_by_helper for getting things through helper keys
// it has remove which can only erase things through helper keys
// initially used to get things though id or a biyection(unique key) and delete through id
// e.g resource's id and resource kind mapped to the resource
#[derive(Debug, Clone)]
pub(crate) struct MultiMap<K1, K2, K3, V> {
    main_map: HashMap<K1, V>,
    helper_map_1: HashMap<K2, (K1, Option<K3>)>,
    helper_map_2: HashMap<K3, (K1, Option<K2>)>,
}

impl<K1, K2, K3, V> Default for MultiMap<K1, K2, K3, V> {
    fn default() -> Self {
        Self {
            main_map: Default::default(),
            helper_map_1: Default::default(),
            helper_map_2: Default::default(),
        }
    }
}

impl<K1, K2, K3, V> MultiMap<K1, K2, K3, V>
where
    K1: Eq + PartialEq + Hash + Clone,
    K2: Eq + PartialEq + Hash + Clone,
    K3: Eq + PartialEq + Hash + Clone,
{
    pub(crate) fn insert(
        &mut self,
        main_key: K1,
        helper_key_1: Option<K2>,
        helper_key_2: Option<K3>,
        value: V,
    ) -> Option<V> {
        if let Some(k) = &helper_key_1 {
            self.helper_map_1
                .insert(k.clone(), (main_key.clone(), helper_key_2.clone()));
        }

        if let Some(k) = helper_key_2 {
            self.helper_map_2
                .insert(k, (main_key.clone(), helper_key_1));
        }
        self.main_map.insert(main_key, value)
    }

    pub(crate) fn get_main(&self, k: &K1) -> Option<&V> {
        self.main_map.get(k)
    }

    pub(crate) fn get_by_helper_1(&self, k: &K2) -> Option<&V> {
        let (k, _) = self.helper_map_1.get(k)?;
        self.main_map.get(k)
    }

    pub(crate) fn get_by_helper_2(&self, k: &K3) -> Option<&V> {
        let (k, _) = self.helper_map_2.get(k)?;
        self.main_map.get(k)
    }

    pub(crate) fn _remove_by_helper_1(&mut self, k: &K2) -> Option<V> {
        let (k, k1) = self.helper_map_1.get(k)?;
        if let Some(k) = k1 {
            self.helper_map_2.remove(k);
        }
        self.main_map.remove(k)
    }

    pub(crate) fn _remove_by_helper_2(&mut self, k: &K3) -> Option<V> {
        let (k, k2) = self.helper_map_2.get(k)?;
        if let Some(k) = k2 {
            self.helper_map_1.remove(k);
        }
        self.main_map.remove(k)
    }
}
