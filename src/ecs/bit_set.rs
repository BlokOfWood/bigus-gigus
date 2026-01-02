use std::u64;

pub struct BitSet {
    memberships: u64,
}

impl BitSet {
    pub fn new() -> Self {
        Self {
           memberships: 0, 
        }
    }

    pub fn add_to_membership(&mut self, membership_id: u8) {
        if membership_id > 63 {
            panic!("BitSet only supports a max of 64 memberships.")
        }

        self.memberships |= 1 << membership_id;
    }

    pub fn remove_from_membership(&mut self, membership_id: u8) {
        if membership_id > 63 {
            panic!("BitSet only supports a max of 64 memberships.")
        }

        self.memberships &= !(1 << membership_id);
    }

    pub fn check_membership(&self, memberships: &[u8]) -> bool {
        for id in memberships {
            if self.memberships & (1 << id) == 0 {
                return false;
            }
        }

        true
    }

}