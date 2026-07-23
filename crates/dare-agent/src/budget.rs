//! Token budget tracker (`0` = unlimited).

#[derive(Debug, Clone)]
pub struct BudgetTracker {
    limit: Option<u64>,
    used: u64,
}

impl BudgetTracker {
    /// `budget_tokens == 0` means unlimited.
    pub fn new(budget_tokens: u64) -> Self {
        Self {
            limit: if budget_tokens == 0 {
                None
            } else {
                Some(budget_tokens)
            },
            used: 0,
        }
    }

    pub fn remaining(&self) -> Option<u64> {
        self.limit.map(|lim| lim.saturating_sub(self.used))
    }

    pub fn can_continue(&self) -> bool {
        match self.limit {
            None => true,
            Some(lim) => self.used < lim,
        }
    }

    pub fn used(&self) -> u64 {
        self.used
    }

    pub fn limit(&self) -> Option<u64> {
        self.limit
    }

    /// Consume tokens. Returns `false` if already exhausted or would exceed a finite limit.
    pub fn consume(&mut self, tokens: u64) -> bool {
        if !self.can_continue() {
            return false;
        }
        match self.limit {
            None => {
                self.used = self.used.saturating_add(tokens);
                true
            }
            Some(lim) => {
                if self.used.saturating_add(tokens) > lim {
                    return false;
                }
                self.used = self.used.saturating_add(tokens);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_zero_unlimited() {
        let mut b = BudgetTracker::new(0);
        assert!(b.can_continue());
        assert!(b.remaining().is_none());
        assert!(b.consume(1_000_000));
        assert!(b.can_continue());
        assert_eq!(b.used(), 1_000_000);
    }

    #[test]
    fn budget_exhaust() {
        let mut b = BudgetTracker::new(2);
        assert!(b.consume(1));
        assert!(b.can_continue());
        assert!(b.consume(1));
        assert!(!b.can_continue());
        assert!(!b.consume(1));
        assert_eq!(b.used(), 2);
        assert_eq!(b.remaining(), Some(0));
    }
}
