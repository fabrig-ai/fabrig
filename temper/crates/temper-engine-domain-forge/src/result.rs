//! What a landed change actually saw (domain/forge/change/README.md, 2).
use alloc::boxed::Box;
use skein_forge::api::{Check, Commit, Status};
use skein_lib::{Decimal, Writer};
use temper_engine_domain_forge_change as change;

/// A bounded UTF-8 prefix and the exact byte count omitted from its end.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ResultText {
    pub bytes: Box<[u8]>,
    pub cut: u64,
}
impl ResultText {
    /// No observations or omitted bytes.
    #[must_use]
    pub fn empty() -> Self {
        Self { bytes: Box::new([]), cut: 0 }
    }

    pub(crate) fn add(&mut self, bytes: &[u8], bound: u32) {
        let room = skein_lib::usize_of(bound).saturating_sub(self.bytes.len());
        let kept = if self.cut == 0 { crate::held::edge(bytes, room.min(bytes.len()), false) } else { &[] };
        let mut writer = Writer::new(self.bytes.len().checked_add(kept.len()).expect("bounded result text"));
        writer.put(&self.bytes).expect("measured prefix");
        writer.put(kept).expect("measured addition");
        self.bytes = writer.finish();
        self.cut =
            self.cut.saturating_add(u64::try_from(bytes.len().saturating_sub(kept.len())).expect("usize fits u64"));
    }
    fn join(&mut self, text: &Self, bound: u32) {
        self.add(&text.bytes, bound);
        self.cut = self.cut.saturating_add(text.cut);
    }
    /// Least room for either the whole text or a mark counting every omitted byte.
    pub(crate) fn minimum_room(&self) -> u64 {
        let length = skein_lib::len_u64(self.bytes.len());
        let lost = self.cut.saturating_add(length);
        let marker = 2_u64
            .saturating_add(skein_lib::len_u64(Decimal::of(lost).as_bytes().len()))
            .saturating_add(skein_lib::len_u64(b" bytes cut]\n".len()));
        if self.cut == 0 { length.min(marker) } else { marker }
    }

    /// Fit from the end, marking every omitted byte. No unmarked partial UTF-8.
    #[must_use]
    pub fn fitted(&self, bound: u32) -> Option<Box<[u8]>> {
        let room = usize::try_from(bound).ok()?;
        if self.cut == 0 && self.bytes.len() <= room {
            return Some(self.bytes.clone());
        }
        let mut kept = crate::held::edge(&self.bytes, self.bytes.len().min(room), false);
        for _ in 0..=room {
            let lost = self.cut.saturating_add(u64::try_from(self.bytes.len().saturating_sub(kept.len())).ok()?);
            let count = Decimal::of(lost);
            let size =
                kept.len().checked_add(2)?.checked_add(count.as_bytes().len())?.checked_add(b" bytes cut]\n".len())?;
            if size <= room {
                let mut writer = Writer::new(size);
                writer.put(kept).ok()?;
                writer.put(b"\n[").ok()?;
                writer.put(count.as_bytes()).ok()?;
                writer.put(b" bytes cut]\n").ok()?;
                return Some(writer.finish());
            }
            if kept.is_empty() {
                return None;
            }
            kept = crate::held::edge(kept, kept.len().saturating_sub(1), false);
        }
        None
    }
}

/// Each text is bounded by the forge client's answer byte bound. Histories
/// retain prefixes across releases; their counts and cuts never reset.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangeResult {
    pub head: Option<Commit>,
    pub checks: ResultText,
    pub failed: ResultText,
    pub repairs: ResultText,
    pub repair_count: u64,
    pub reviews: ResultText,
}
impl ChangeResult {
    /// A newly admitted change has seen no checks, repairs or reviews.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            head: None,
            checks: ResultText::empty(),
            failed: ResultText::empty(),
            repairs: ResultText::empty(),
            repair_count: 0,
            reviews: ResultText::empty(),
        }
    }

    pub(crate) fn checked(&mut self, head: Commit, checks: &[Status], bound: u32) -> bool {
        let mut checks_text = ResultText::empty();
        let mut failed = ResultText::empty();
        for check in checks {
            if !checks_text.bytes.is_empty() || checks_text.cut != 0 {
                checks_text.add(b"; ", bound);
            }
            checks_text.add(&check.context, bound);
            checks_text.add(
                match check.check {
                    Check::Passed => b" passed",
                    Check::Failed => b" failed",
                    Check::Pending => b" pending",
                },
                bound,
            );
            if check.check != Check::Pending
                && let Some(job) = check.job
                && job.head == head
                && job.attempt > 1
            {
                checks_text.add(b" at attempt ", bound);
                checks_text.add(Decimal::of(u64::from(job.attempt)).as_bytes(), bound);
            }
            if check.check == Check::Failed {
                if !failed.bytes.is_empty() || failed.cut != 0 {
                    failed.add(b", ", bound);
                }
                failed.add(&check.context, bound);
            }
        }
        if self.head == Some(head) && self.checks == checks_text && self.failed == failed {
            return false;
        }
        self.head = Some(head);
        self.checks = checks_text;
        self.failed = failed;
        true
    }
    pub(crate) fn repaired(&mut self, why: change::Repair, head: Option<Commit>, bound: u32) {
        if self.repair_count != 0 {
            self.repairs.add(b"; ", bound);
        }
        self.repair_count = self.repair_count.saturating_add(1);
        match why {
            change::Repair::Ci => {
                self.repairs.add(b"CI failed", bound);
                if self.head == head && (!self.failed.bytes.is_empty() || self.failed.cut != 0) {
                    self.repairs.add(b": ", bound);
                    self.repairs.join(&self.failed, bound);
                }
            }
            change::Repair::Review { .. } => self.repairs.add(b"a person asked for changes on the forge", bound),
            change::Repair::Gate(_) => self.repairs.add(b"review asked for changes", bound),
            change::Repair::Semantic => self.repairs.add(b"the base broke it", bound),
            change::Repair::Landing { gate } => {
                self.repairs.add(b"landing gate ", bound);
                self.repairs.add(Decimal::of(u64::from(gate)).as_bytes(), bound);
                self.repairs.add(b" failed", bound);
            }
        }
    }
    /// A CI repair ended by a green head (domain/forge/change/repairs.md). Nothing to say where none was asked.
    pub(crate) fn ci_passed(&mut self, head: &Commit, bound: u32) {
        if self.repair_count == 0 {
            return;
        }
        self.repairs.add(b", then CI passed at ", bound);
        self.repairs.add(&crate::commit_text(head), bound);
    }
    pub(crate) fn reviewed(&mut self, head: &Commit, verdict: &[u8], bound: u32) {
        if !self.reviews.bytes.is_empty() || self.reviews.cut != 0 {
            self.reviews.add(b"; ", bound);
        }
        self.reviews.add(verdict, bound);
        self.reviews.add(b" at ", bound);
        self.reviews.add(&crate::commit_text(head), bound);
    }
    /// Retain a successful repair account with its task identity in completion order.
    pub(crate) fn completed(&mut self, child: u64, report: &[u8], bound: u32) {
        self.repairs.add(b"\nsuccessful repair task ", bound);
        self.repairs.add(Decimal::of(child).as_bytes(), bound);
        self.repairs.add(b": ", bound);
        self.repairs.add(report, bound);
    }
    /// Repair accounts precede the producer's historical report, then the landing's observations; cut from the end.
    #[must_use]
    pub fn reported(&self, report: &[u8], head: &Commit, bound: u32) -> ResultText {
        let mut words = ResultText::empty();
        if self.repair_count != 0 {
            self.repair_words(&mut words, bound);
            words.add(b"\n", bound);
        }
        if !report.is_empty() {
            words.add(
                if self.repair_count == 0 { b"Report:\n" } else { b"Producer report (historical; before repairs):\n" },
                bound,
            );
            words.add(report, bound);
            words.add(b"\n", bound);
        }
        if self.repair_count == 0 {
            words.join(&self.words(head, bound), bound);
        } else {
            self.check_words(&mut words, head, bound);
            self.review_words(&mut words, bound);
        }
        words
    }

    /// Result suffix, including only checks read at the head that landed.
    #[must_use]
    pub fn words(&self, head: &Commit, bound: u32) -> ResultText {
        let mut words = ResultText::empty();
        self.check_words(&mut words, head, bound);
        words.add(b"\n", bound);
        self.repair_words(&mut words, bound);
        self.review_words(&mut words, bound);
        words
    }

    fn check_words(&self, words: &mut ResultText, head: &Commit, bound: u32) {
        words.add(b"checks at ", bound);
        words.add(&crate::commit_text(head), bound);
        words.add(b": ", bound);
        if self.head == Some(*head) && (!self.checks.bytes.is_empty() || self.checks.cut != 0) {
            words.join(&self.checks, bound);
        } else {
            words.add(b"none", bound);
        }
    }

    fn repair_words(&self, words: &mut ResultText, bound: u32) {
        words.add(b"repairs: ", bound);
        words.add(Decimal::of(self.repair_count).as_bytes(), bound);
        if self.repair_count != 0 {
            words.add(b": ", bound);
            words.join(&self.repairs, bound);
        }
    }

    fn review_words(&self, words: &mut ResultText, bound: u32) {
        words.add(b"\nreviews: ", bound);
        if self.reviews.bytes.is_empty() && self.reviews.cut == 0 {
            words.add(b"none", bound);
        } else {
            words.join(&self.reviews, bound);
        }
    }
}

#[cfg(test)]
#[path = "result_tests.rs"]
mod tests;
