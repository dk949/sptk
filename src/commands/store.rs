use crate::commands::{Result, Value};

/// Holds outputs of pipeline steps that are referenced by `$N` / `${N}`.
/// Indexed by step number: 0 = original input, 1..=n = output of each
/// step-producing cmd. Slots not referenced by any cmd stay `None`.
pub struct StepStore {
    slots: Vec<Option<Value>>,
}

impl StepStore {
    pub fn new(len: usize) -> Self {
        Self {
            slots: (0..len).map(|_| None).collect(),
        }
    }

    pub fn set(&mut self, idx: usize, v: Value) {
        self.slots[idx] = Some(v);
    }

    pub fn get(&self, idx: usize) -> Result<&Value> {
        match self.slots.get(idx) {
            Some(Some(v)) => Ok(v),
            Some(None) => Err(format!(
                "step {idx} was not stored (internal: missing from pre-scan)"
            )),
            None => Err(format!("step {idx} out of bounds")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_slot_errors() {
        let s = StepStore::new(3);
        assert!(s.get(0).is_err());
        assert!(s.get(1).is_err());
    }

    #[test]
    fn set_then_get() {
        let mut s = StepStore::new(3);
        s.set(1, Value::String("hi".into()));
        assert!(matches!(s.get(1).unwrap(), Value::String(x) if x == "hi"));
    }

    #[test]
    fn out_of_bounds_errors() {
        let s = StepStore::new(2);
        assert!(s.get(2).is_err());
        assert!(s.get(99).is_err());
    }
}
