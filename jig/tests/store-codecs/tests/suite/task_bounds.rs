//! The tasks family's bounds (skein's data/codec/README.md, 4).
//! Decoded literals pin the generated types' sizes on a 64-bit target.

use jig_protocol_store::tasks_v13::*;

#[test]
#[expect(clippy::too_many_lines, reason = "One literal pair per declaration, in schema order.")]
#[rustfmt::skip] // Keep each declaration's two calls and literal bounds on one line.
fn every_tasks_declaration_keeps_its_ceiling_bounds() {
    let limits = CEILINGS;
    assert_eq!((Number::encoded_bound(&limits), Number::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Person::encoded_bound(&limits), Person::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Project::encoded_bound(&limits), Project::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Party::encoded_bound(&limits), Party::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Class::encoded_bound(&limits), Class::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Tries::encoded_bound(&limits), Tries::decoded_bound(&limits)), (Some(28), Some(0)));
    assert_eq!((Why::encoded_bound(&limits), Why::decoded_bound(&limits)), (Some(29), Some(0)));
    assert_eq!((Role::encoded_bound(&limits), Role::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((PersonAddress::encoded_bound(&limits), PersonAddress::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Agent::encoded_bound(&limits), Agent::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Procedure::encoded_bound(&limits), Procedure::decoded_bound(&limits)), (Some(6), Some(0)));
    assert_eq!((PersonExecutor::encoded_bound(&limits), PersonExecutor::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((JobExecutor::encoded_bound(&limits), JobExecutor::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Executor::encoded_bound(&limits), Executor::decoded_bound(&limits)), (Some(10), Some(0)));
    assert_eq!((NumericParameter::encoded_bound(&limits), NumericParameter::decoded_bound(&limits)), (Some(12), Some(0)));
    assert_eq!((BytesParameter::encoded_bound(&limits), BytesParameter::decoded_bound(&limits)), (Some(262_152), Some(262_144)));
    assert_eq!((ResourceParameter::encoded_bound(&limits), ResourceParameter::decoded_bound(&limits)), (Some(14), Some(0)));
    assert_eq!((Parameter::encoded_bound(&limits), Parameter::decoded_bound(&limits)), (Some(262_153), Some(262_144)));
    assert_eq!((JobSpec::encoded_bound(&limits), JobSpec::decoded_bound(&limits)), (Some(262_156), Some(262_144)));
    assert_eq!((JobContract::encoded_bound(&limits), JobContract::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((JobResult::encoded_bound(&limits), JobResult::decoded_bound(&limits)), (Some(524_292), Some(524_288)));
    assert_eq!((Spec::encoded_bound(&limits), Spec::decoded_bound(&limits)), (Some(1_573_402), Some(1_573_376)));
    assert_eq!((PartKind::encoded_bound(&limits), PartKind::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((PartSegment::encoded_bound(&limits), PartSegment::decoded_bound(&limits)), (Some(4_100), Some(4_096)));
    assert_eq!((PartPath::encoded_bound(&limits), PartPath::decoded_bound(&limits)), (Some(65_540), Some(65_536)));
    assert_eq!((PartKey::encoded_bound(&limits), PartKey::decoded_bound(&limits)), (Some(65_541), Some(65_536)));
    assert_eq!((Part::encoded_bound(&limits), Part::decoded_bound(&limits)), (Some(65_544), Some(65_536)));
    assert_eq!((FieldRule::encoded_bound(&limits), FieldRule::decoded_bound(&limits)), (Some(524_296), Some(524_288)));
    assert_eq!((Verdict::encoded_bound(&limits), Verdict::decoded_bound(&limits)), (Some(1_048_592), Some(1_048_576)));
    assert_eq!((Approval::encoded_bound(&limits), Approval::decoded_bound(&limits)), (Some(589_836), Some(589_824)));
    assert_eq!((Report::encoded_bound(&limits), Report::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Choices::encoded_bound(&limits), Choices::decoded_bound(&limits)), (Some(67_109_892), Some(67_110_912)));
    assert_eq!((ChangeContract::encoded_bound(&limits), ChangeContract::decoded_bound(&limits)), (Some(67_109_900), Some(67_110_912)));
    assert_eq!((Contract::encoded_bound(&limits), Contract::decoded_bound(&limits)), (Some(67_109_901), Some(67_110_912)));
    assert_eq!((Tools::encoded_bound(&limits), Tools::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Models::encoded_bound(&limits), Models::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((ModelEffort::encoded_bound(&limits), ModelEffort::decoded_bound(&limits)), (Some(3), Some(0)));
    assert_eq!((LlmLater::encoded_bound(&limits), LlmLater::decoded_bound(&limits)), (Some(199), Some(192)));
    assert_eq!((LlmChoice::encoded_bound(&limits), LlmChoice::decoded_bound(&limits)), (Some(404), Some(384)));
    assert_eq!((Scopes::encoded_bound(&limits), Scopes::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((AuthoritySegment::encoded_bound(&limits), AuthoritySegment::decoded_bound(&limits)), (Some(4_100), Some(4_096)));
    assert_eq!((Last::encoded_bound(&limits), Last::decoded_bound(&limits)), (Some(4_101), Some(4_096)));
    assert_eq!((Pattern::encoded_bound(&limits), Pattern::decoded_bound(&limits)), (Some(266_505), Some(267_264)));
    assert_eq!((Grant::encoded_bound(&limits), Grant::decoded_bound(&limits)), (Some(266_509), Some(267_264)));
    assert_eq!((ResourceScope::encoded_bound(&limits), ResourceScope::decoded_bound(&limits)), (Some(266_507), Some(267_264)));
    assert_eq!((Charter::encoded_bound(&limits), Charter::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Code::encoded_bound(&limits), Code::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((AuthorityJob::encoded_bound(&limits), AuthorityJob::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((AuthorityExecutor::encoded_bound(&limits), AuthorityExecutor::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((Delegation::encoded_bound(&limits), Delegation::decoded_bound(&limits)), (Some(332), Some(512)));
    assert_eq!((Budget::encoded_bound(&limits), Budget::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((Authority::encoded_bound(&limits), Authority::decoded_bound(&limits)), (Some(131_447), Some(131_584)));
    assert_eq!((Task::encoded_bound(&limits), Task::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Pool::encoded_bound(&limits), Pool::decoded_bound(&limits)), (Some(20), Some(0)));
    assert_eq!((Recurring::encoded_bound(&limits), Recurring::decoded_bound(&limits)), (Some(20), Some(0)));
    assert_eq!((Period::encoded_bound(&limits), Period::decoded_bound(&limits)), (Some(12), Some(0)));
    assert_eq!((Funder::encoded_bound(&limits), Funder::decoded_bound(&limits)), (Some(21), Some(0)));
    assert_eq!((Numbers::encoded_bound(&limits), Numbers::decoded_bound(&limits)), (Some(32), Some(0)));
    assert_eq!((PeriodTimes::encoded_bound(&limits), PeriodTimes::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((FundingRecord::encoded_bound(&limits), FundingRecord::decoded_bound(&limits)), (Some(98), Some(0)));
    assert_eq!((Segment::encoded_bound(&limits), Segment::decoded_bound(&limits)), (Some(4_100), Some(4_096)));
    assert_eq!((Name::encoded_bound(&limits), Name::decoded_bound(&limits)), (Some(65_542), Some(65_536)));
    assert_eq!((Run::encoded_bound(&limits), Run::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((Effect::encoded_bound(&limits), Effect::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Writer::encoded_bound(&limits), Writer::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((WriterSlot::encoded_bound(&limits), WriterSlot::decoded_bound(&limits)), (Some(65_568), Some(65_536)));
    assert_eq!((PoolSlots::encoded_bound(&limits), PoolSlots::decoded_bound(&limits)), (Some(65_554), Some(65_536)));
    assert_eq!((WriteHolding::encoded_bound(&limits), WriteHolding::decoded_bound(&limits)), (Some(65_544), Some(65_536)));
    assert_eq!((SlotHolding::encoded_bound(&limits), SlotHolding::decoded_bound(&limits)), (Some(65_544), Some(65_536)));
    assert_eq!((Holding::encoded_bound(&limits), Holding::decoded_bound(&limits)), (Some(65_545), Some(65_536)));
    assert_eq!((WakeBatch::encoded_bound(&limits), WakeBatch::decoded_bound(&limits)), (Some(12), Some(0)));
    assert_eq!((WakePace::encoded_bound(&limits), WakePace::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((WakeRule::encoded_bound(&limits), WakeRule::decoded_bound(&limits)), (Some(13), Some(0)));
    assert_eq!((ResultsWake::encoded_bound(&limits), ResultsWake::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((WakePolicy::encoded_bound(&limits), WakePolicy::decoded_bound(&limits)), (Some(43), Some(0)));
    assert_eq!((RecurringOverlap::encoded_bound(&limits), RecurringOverlap::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Retention::encoded_bound(&limits), Retention::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Closer::encoded_bound(&limits), Closer::decoded_bound(&limits)), (Some(6), Some(0)));
    assert_eq!((EndedBy::encoded_bound(&limits), EndedBy::decoded_bound(&limits)), (Some(7), Some(0)));
    assert_eq!((Flags::encoded_bound(&limits), Flags::decoded_bound(&limits)), (Some(10), Some(0)));
    assert_eq!((DeadlineSpan::encoded_bound(&limits), DeadlineSpan::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Deadline::encoded_bound(&limits), Deadline::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Member::encoded_bound(&limits), Member::decoded_bound(&limits)), (Some(73_010_701), Some(73_013_632)));
    assert_eq!((RecurringTemplate::encoded_bound(&limits), RecurringTemplate::decoded_bound(&limits)), (Some(67_108_873), Some(67_108_864)));
    assert_eq!((Creation::encoded_bound(&limits), Creation::decoded_bound(&limits)), (Some(140_119_575), Some(140_122_496)));
    assert_eq!((ResultField::encoded_bound(&limits), ResultField::decoded_bound(&limits)), (Some(1_048_584), Some(1_048_576)));
    assert_eq!((Approved::encoded_bound(&limits), Approved::decoded_bound(&limits)), (Some(1_048_585), Some(1_048_576)));
    assert_eq!((ReportResult::encoded_bound(&limits), ReportResult::decoded_bound(&limits)), (Some(524_292), Some(524_288)));
    assert_eq!((VerdictResult::encoded_bound(&limits), VerdictResult::decoded_bound(&limits)), (Some(1_572_876), Some(1_572_864)));
    assert_eq!((ChangeResult::encoded_bound(&limits), ChangeResult::decoded_bound(&limits)), (Some(1_048_596), Some(1_048_576)));
    assert_eq!((FailureResult::encoded_bound(&limits), FailureResult::decoded_bound(&limits)), (Some(524_292), Some(524_288)));
    assert_eq!((TaskResult::encoded_bound(&limits), TaskResult::decoded_bound(&limits)), (Some(1_572_877), Some(1_572_864)));
    assert_eq!((Done::encoded_bound(&limits), Done::decoded_bound(&limits)), (Some(1_572_877), Some(1_572_864)));
    assert_eq!((Cancelled::encoded_bound(&limits), Cancelled::decoded_bound(&limits)), (Some(2_097_187), Some(2_097_152)));
    assert_eq!((Ending::encoded_bound(&limits), Ending::decoded_bound(&limits)), (Some(2_097_188), Some(2_097_152)));
    assert_eq!((FailedClass::encoded_bound(&limits), FailedClass::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Bound::encoded_bound(&limits), Bound::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Exceeded::encoded_bound(&limits), Exceeded::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Hold::encoded_bound(&limits), Hold::decoded_bound(&limits)), (Some(10), Some(0)));
    assert_eq!((Active::encoded_bound(&limits), Active::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Stage::encoded_bound(&limits), Stage::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Closing::encoded_bound(&limits), Closing::decoded_bound(&limits)), (Some(2_097_197), Some(2_097_152)));
    assert_eq!((Activation::encoded_bound(&limits), Activation::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Was::encoded_bound(&limits), Was::decoded_bound(&limits)), (Some(2_097_198), Some(2_097_152)));
    assert_eq!((HoldEffect::encoded_bound(&limits), HoldEffect::decoded_bound(&limits)), (Some(10), Some(0)));
    assert_eq!((Circle::encoded_bound(&limits), Circle::decoded_bound(&limits)), (Some(261), Some(256)));
    assert_eq!((HoldAccount::encoded_bound(&limits), HoldAccount::decoded_bound(&limits)), (Some(527), Some(512)));
    assert_eq!((HoldPreparation::encoded_bound(&limits), HoldPreparation::decoded_bound(&limits)), (Some(132), Some(128)));
    assert_eq!((HoldSource::encoded_bound(&limits), HoldSource::decoded_bound(&limits)), (Some(65_543), Some(65_536)));
    assert_eq!((Held::encoded_bound(&limits), Held::decoded_bound(&limits)), (Some(2_162_752), Some(2_162_688)));
    assert_eq!((Final::encoded_bound(&limits), Final::decoded_bound(&limits)), (Some(2_097_188), Some(2_097_152)));
    assert_eq!((Phase::encoded_bound(&limits), Phase::decoded_bound(&limits)), (Some(2_162_753), Some(2_162_688)));
    assert_eq!((ProposalKind::encoded_bound(&limits), ProposalKind::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((PolicyHolder::encoded_bound(&limits), PolicyHolder::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((ProposalHolder::encoded_bound(&limits), ProposalHolder::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((PendingProposal::encoded_bound(&limits), PendingProposal::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((AcceptedProposal::encoded_bound(&limits), AcceptedProposal::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((RejectedProposal::encoded_bound(&limits), RejectedProposal::decoded_bound(&limits)), (Some(524_301), Some(524_288)));
    assert_eq!((ProposalState::encoded_bound(&limits), ProposalState::decoded_bound(&limits)), (Some(524_302), Some(524_288)));
    assert_eq!((AddressChange::encoded_bound(&limits), AddressChange::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((Tracked::encoded_bound(&limits), Tracked::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Tracking::encoded_bound(&limits), Tracking::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((Amendment::encoded_bound(&limits), Amendment::decoded_bound(&limits)), (Some(2_754_435), Some(2_754_432)));
    assert_eq!((ProposedBatch::encoded_bound(&limits), ProposedBatch::decoded_bound(&limits)), (Some(67_108_868), Some(67_108_864)));
    assert_eq!((ProposedEffect::encoded_bound(&limits), ProposedEffect::decoded_bound(&limits)), (Some(131_473), Some(131_584)));
    assert_eq!((ProposedProcedureEffect::encoded_bound(&limits), ProposedProcedureEffect::decoded_bound(&limits)), (Some(131_465), Some(131_584)));
    assert_eq!((ProposedAmendment::encoded_bound(&limits), ProposedAmendment::decoded_bound(&limits)), (Some(2_754_451), Some(2_754_432)));
    assert_eq!((ProposedWidening::encoded_bound(&limits), ProposedWidening::decoded_bound(&limits)), (Some(131_455), Some(131_584)));
    assert_eq!((ReleasePlaced::encoded_bound(&limits), ReleasePlaced::decoded_bound(&limits)), (Some(24), Some(0)));
    assert_eq!((SettingNumber::encoded_bound(&limits), SettingNumber::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((Setting::encoded_bound(&limits), Setting::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((SettingEdit::encoded_bound(&limits), SettingEdit::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((SettingHead::encoded_bound(&limits), SettingHead::decoded_bound(&limits)), (Some(6), Some(0)));
    assert_eq!((ProposedSettings::encoded_bound(&limits), ProposedSettings::decoded_bound(&limits)), (Some(908), Some(1_664)));
    assert_eq!((ProposalAction::encoded_bound(&limits), ProposalAction::decoded_bound(&limits)), (Some(67_108_869), Some(67_108_864)));
    assert_eq!((Proposal::encoded_bound(&limits), Proposal::decoded_bound(&limits)), (Some(68_157_484), Some(68_157_440)));
    assert_eq!((Change::encoded_bound(&limits), Change::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Settings::encoded_bound(&limits), Settings::decoded_bound(&limits)), (Some(2_230_130), Some(2_230_144)));
    assert_eq!((AmendmentHistory::encoded_bound(&limits), AmendmentHistory::decoded_bound(&limits)), (Some(4_460_260), Some(4_460_288)));
    assert_eq!((History::encoded_bound(&limits), History::decoded_bound(&limits)), (Some(73_142_072), Some(73_142_016)));
    assert_eq!((PendingPersonProposal::encoded_bound(&limits), PendingPersonProposal::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((RejectedPersonProposal::encoded_bound(&limits), RejectedPersonProposal::decoded_bound(&limits)), (Some(524_317), Some(524_288)));
    assert_eq!((WithdrawnPersonProposal::encoded_bound(&limits), WithdrawnPersonProposal::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((SettingsAccepted::encoded_bound(&limits), SettingsAccepted::decoded_bound(&limits)), (Some(13), Some(0)));
    assert_eq!((PersonProposalState::encoded_bound(&limits), PersonProposalState::decoded_bound(&limits)), (Some(524_318), Some(524_288)));
    assert_eq!((PersonAction::encoded_bound(&limits), PersonAction::decoded_bound(&limits)), (Some(67_108_869), Some(67_108_864)));
    assert_eq!((PersonProposal::encoded_bound(&limits), PersonProposal::decoded_bound(&limits)), (Some(68_157_499), Some(68_157_440)));
    assert_eq!((HoldRow::encoded_bound(&limits), HoldRow::decoded_bound(&limits)), (Some(2_162_776), Some(2_162_688)));
    assert_eq!((Grace::encoded_bound(&limits), Grace::decoded_bound(&limits)), (Some(524_317), Some(524_288)));
    assert_eq!((SettingsRevision::encoded_bound(&limits), SettingsRevision::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Revisions::encoded_bound(&limits), Revisions::decoded_bound(&limits)), (Some(24), Some(0)));
    assert_eq!((FailedRetry::encoded_bound(&limits), FailedRetry::decoded_bound(&limits)), (Some(142), Some(128)));
    assert_eq!((RefusedRetry::encoded_bound(&limits), RefusedRetry::decoded_bound(&limits)), (Some(137), Some(128)));
    assert_eq!((RetryNotice::encoded_bound(&limits), RetryNotice::decoded_bound(&limits)), (Some(143), Some(128)));
    assert_eq!((NoticeHeld::encoded_bound(&limits), NoticeHeld::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((NoticeState::encoded_bound(&limits), NoticeState::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((NewsClass::encoded_bound(&limits), NewsClass::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((ApprovalMessage::encoded_bound(&limits), ApprovalMessage::decoded_bound(&limits)), (Some(524_293), Some(524_288)));
    assert_eq!((VerdictMessage::encoded_bound(&limits), VerdictMessage::decoded_bound(&limits)), (Some(1_048_584), Some(1_048_576)));
    assert_eq!((ChangeMessage::encoded_bound(&limits), ChangeMessage::decoded_bound(&limits)), (Some(12), Some(0)));
    assert_eq!((ResultKind::encoded_bound(&limits), ResultKind::decoded_bound(&limits)), (Some(1_048_585), Some(1_048_576)));
    assert_eq!((EscalationMessage::encoded_bound(&limits), EscalationMessage::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((ProposalMessage::encoded_bound(&limits), ProposalMessage::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((StalledMessage::encoded_bound(&limits), StalledMessage::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((DecisionMessage::encoded_bound(&limits), DecisionMessage::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((SettingsAcceptedRevision::encoded_bound(&limits), SettingsAcceptedRevision::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((SettingsStaleRevision::encoded_bound(&limits), SettingsStaleRevision::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((SettingsOutcome::encoded_bound(&limits), SettingsOutcome::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((SettingsDecisionMessage::encoded_bound(&limits), SettingsDecisionMessage::decoded_bound(&limits)), (Some(13), Some(0)));
    assert_eq!((EffectMessage::encoded_bound(&limits), EffectMessage::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((AmendmentMessage::encoded_bound(&limits), AmendmentMessage::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((AnswerMessage::encoded_bound(&limits), AnswerMessage::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((NoticeMessage::encoded_bound(&limits), NoticeMessage::decoded_bound(&limits)), (Some(25), Some(0)));
    assert_eq!((TimerMessage::encoded_bound(&limits), TimerMessage::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((NewsMessage::encoded_bound(&limits), NewsMessage::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((ResultMessage::encoded_bound(&limits), ResultMessage::decoded_bound(&limits)), (Some(1_048_585), Some(1_048_576)));
    assert_eq!((MetMessage::encoded_bound(&limits), MetMessage::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((MessageKind::encoded_bound(&limits), MessageKind::decoded_bound(&limits)), (Some(1_048_586), Some(1_048_576)));
    assert_eq!((Word::encoded_bound(&limits), Word::decoded_bound(&limits)), (Some(1_572_908), Some(1_572_864)));
    assert_eq!((InvalidResult::encoded_bound(&limits), InvalidResult::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((FailedEnd::encoded_bound(&limits), FailedEnd::decoded_bound(&limits)), (Some(3), Some(0)));
    assert_eq!((EndKind::encoded_bound(&limits), EndKind::decoded_bound(&limits)), (Some(6), Some(0)));
    assert_eq!((AttemptRow::encoded_bound(&limits), AttemptRow::decoded_bound(&limits)), (Some(55), Some(0)));
    assert_eq!((OpenAttempt::encoded_bound(&limits), OpenAttempt::decoded_bound(&limits)), (Some(41), Some(0)));
    assert_eq!((ActivityWords::encoded_bound(&limits), ActivityWords::decoded_bound(&limits)), (Some(1_572_895), Some(1_572_864)));
    assert_eq!((HeardMessage::encoded_bound(&limits), HeardMessage::decoded_bound(&limits)), (Some(1_572_908), Some(1_572_864)));
    assert_eq!((NotifiedMessage::encoded_bound(&limits), NotifiedMessage::decoded_bound(&limits)), (Some(1_572_917), Some(1_572_864)));
    assert_eq!((Fresh::encoded_bound(&limits), Fresh::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((BeganLine::encoded_bound(&limits), BeganLine::decoded_bound(&limits)), (Some(40), Some(0)));
    assert_eq!((EndedLine::encoded_bound(&limits), EndedLine::decoded_bound(&limits)), (Some(22), Some(0)));
    assert_eq!((TurnLine::encoded_bound(&limits), TurnLine::decoded_bound(&limits)), (Some(29), Some(0)));
    assert_eq!((DecidedProposal::encoded_bound(&limits), DecidedProposal::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((DecidedLine::encoded_bound(&limits), DecidedLine::decoded_bound(&limits)), (Some(18), Some(0)));
    assert_eq!((MilestoneLine::encoded_bound(&limits), MilestoneLine::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((KeptLine::encoded_bound(&limits), KeptLine::decoded_bound(&limits)), (Some(533), Some(512)));
    assert_eq!((NowWrittenLine::encoded_bound(&limits), NowWrittenLine::decoded_bound(&limits)), (Some(524), Some(512)));
    assert_eq!((StoodLine::encoded_bound(&limits), StoodLine::decoded_bound(&limits)), (Some(524), Some(512)));
    assert_eq!((SessionEnd::encoded_bound(&limits), SessionEnd::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((SessionEndedLine::encoded_bound(&limits), SessionEndedLine::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((TalkOpenedLine::encoded_bound(&limits), TalkOpenedLine::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((TalkEndedLine::encoded_bound(&limits), TalkEndedLine::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((Line::encoded_bound(&limits), Line::decoded_bound(&limits)), (Some(1_572_918), Some(1_572_864)));
    assert_eq!((ActivityLine::encoded_bound(&limits), ActivityLine::decoded_bound(&limits)), (Some(1_572_942), Some(1_572_864)));
    assert_eq!((Status::encoded_bound(&limits), Status::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((Stub::encoded_bound(&limits), Stub::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((ReplyHint::encoded_bound(&limits), ReplyHint::decoded_bound(&limits)), (Some(36), Some(0)));
    assert_eq!((EscalationTask::encoded_bound(&limits), EscalationTask::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((EscalationPerson::encoded_bound(&limits), EscalationPerson::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((EscalationRole::encoded_bound(&limits), EscalationRole::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((EscalationHolder::encoded_bound(&limits), EscalationHolder::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((UnheldEscalation::encoded_bound(&limits), UnheldEscalation::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((RoutingEscalation::encoded_bound(&limits), RoutingEscalation::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((WaitingEscalation::encoded_bound(&limits), WaitingEscalation::decoded_bound(&limits)), (Some(33), Some(0)));
    assert_eq!((DeciderKind::encoded_bound(&limits), DeciderKind::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((RejectedEscalation::encoded_bound(&limits), RejectedEscalation::decoded_bound(&limits)), (Some(524_309), Some(524_288)));
    assert_eq!((Escalation::encoded_bound(&limits), Escalation::decoded_bound(&limits)), (Some(524_310), Some(524_288)));
    assert_eq!((PersonAnswer::encoded_bound(&limits), PersonAnswer::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((PersonStallNotice::encoded_bound(&limits), PersonStallNotice::decoded_bound(&limits)), (Some(24), Some(0)));
    assert_eq!((PersonWait::encoded_bound(&limits), PersonWait::decoded_bound(&limits)), (Some(49), Some(0)));
    assert_eq!((RecurringState::encoded_bound(&limits), RecurringState::decoded_bound(&limits)), (Some(67_117_086), Some(67_117_056)));
    assert_eq!((QuestionCredit::encoded_bound(&limits), QuestionCredit::decoded_bound(&limits)), (Some(34), Some(0)));
    assert_eq!((TaskInterest::encoded_bound(&limits), TaskInterest::decoded_bound(&limits)), (Some(10), Some(0)));
    assert_eq!((TimerInterest::encoded_bound(&limits), TimerInterest::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((TopicInterest::encoded_bound(&limits), TopicInterest::decoded_bound(&limits)), (Some(10), Some(0)));
    assert_eq!((SubscriptionKind::encoded_bound(&limits), SubscriptionKind::decoded_bound(&limits)), (Some(18), Some(0)));
    assert_eq!((Subscription::encoded_bound(&limits), Subscription::decoded_bound(&limits)), (Some(26), Some(0)));
    assert_eq!((SavedResource::encoded_bound(&limits), SavedResource::decoded_bound(&limits)), (Some(65_542), Some(65_536)));
    assert_eq!((AnswerLine::encoded_bound(&limits), AnswerLine::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((QuestionLine::encoded_bound(&limits), QuestionLine::decoded_bound(&limits)), (Some(33), Some(0)));
    assert_eq!((ProposalRow::encoded_bound(&limits), ProposalRow::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((Endings::encoded_bound(&limits), Endings::decoded_bound(&limits)), (Some(12), Some(0)));
    assert_eq!((ReplyObject::encoded_bound(&limits), ReplyObject::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((ProposalUpdate::encoded_bound(&limits), ProposalUpdate::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((UpdateObject::encoded_bound(&limits), UpdateObject::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((Update::encoded_bound(&limits), Update::decoded_bound(&limits)), (Some(41), Some(0)));
    assert_eq!((ResultRow::encoded_bound(&limits), ResultRow::decoded_bound(&limits)), (Some(24), Some(0)));
    assert_eq!((Edge::encoded_bound(&limits), Edge::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((TranscriptState::encoded_bound(&limits), TranscriptState::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((TaskHistory::encoded_bound(&limits), TaskHistory::decoded_bound(&limits)), (Some(1_056_195), Some(1_056_896)));
    assert_eq!((TaskRun::encoded_bound(&limits), TaskRun::decoded_bound(&limits)), (Some(68_686_588), Some(68_686_848)));
    assert_eq!((Added::encoded_bound(&limits), Added::decoded_bound(&limits)), (Some(25), Some(0)));
    assert_eq!((TaskDefinition::encoded_bound(&limits), TaskDefinition::decoded_bound(&limits)), (Some(135_935_053), Some(135_936_384)));
    assert_eq!((InboxItem::encoded_bound(&limits), InboxItem::decoded_bound(&limits)), (Some(1_572_909), Some(1_572_864)));
    assert_eq!((TaskInterests::encoded_bound(&limits), TaskInterests::decoded_bound(&limits)), (Some(212_880_261), Some(212_891_344)));
    assert_eq!((RoomProject::encoded_bound(&limits), RoomProject::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((LiveRoom::encoded_bound(&limits), LiveRoom::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((RoomWait::encoded_bound(&limits), RoomWait::decoded_bound(&limits)), (Some(13), Some(0)));
    assert_eq!((TaskOutcome::encoded_bound(&limits), TaskOutcome::decoded_bound(&limits)), (Some(2_162_818), Some(2_162_688)));
    assert_eq!((Talk::encoded_bound(&limits), Talk::decoded_bound(&limits)), (Some(24), Some(0)));
    assert_eq!((Session::encoded_bound(&limits), Session::decoded_bound(&limits)), (Some(20), Some(0)));
    assert_eq!((BeforeTalk::encoded_bound(&limits), BeforeTalk::decoded_bound(&limits)), (Some(420_721_479), Some(420_734_672)));
    assert_eq!((Concluded::encoded_bound(&limits), Concluded::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((TaskRecord::encoded_bound(&limits), TaskRecord::decoded_bound(&limits)), (Some(420_721_521), Some(420_734_672)));
    assert_eq!((HotHistory::encoded_bound(&limits), HotHistory::decoded_bound(&limits)), (Some(531_902), Some(532_608)));
    assert_eq!((HotDefinition::encoded_bound(&limits), HotDefinition::decoded_bound(&limits)), (Some(67_120_303), Some(67_120_512)));
    assert_eq!((LiveHot::encoded_bound(&limits), LiveHot::decoded_bound(&limits)), (Some(351_382_478), Some(351_394_512)));
    assert_eq!((LiveCold::encoded_bound(&limits), LiveCold::decoded_bound(&limits)), (Some(69_339_043), Some(69_340_160)));
    assert_eq!((EndedByParty::encoded_bound(&limits), EndedByParty::decoded_bound(&limits)), (Some(28), Some(0)));
    assert_eq!((EndedByProject::encoded_bound(&limits), EndedByProject::decoded_bound(&limits)), (Some(24), Some(0)));
    assert_eq!((HoldOwner::encoded_bound(&limits), HoldOwner::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((HoldMode::encoded_bound(&limits), HoldMode::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((SubtreeScope::encoded_bound(&limits), SubtreeScope::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((HoldScope::encoded_bound(&limits), HoldScope::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((PlacedHold::encoded_bound(&limits), PlacedHold::decoded_bound(&limits)), (Some(557_112), Some(557_056)));
    assert_eq!((SummaryCancelled::encoded_bound(&limits), SummaryCancelled::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((SummaryEnding::encoded_bound(&limits), SummaryEnding::decoded_bound(&limits)), (Some(18), Some(0)));
    assert_eq!((SummaryVerdict::encoded_bound(&limits), SummaryVerdict::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((SummaryChange::encoded_bound(&limits), SummaryChange::decoded_bound(&limits)), (Some(12), Some(0)));
    assert_eq!((SummaryResultKind::encoded_bound(&limits), SummaryResultKind::decoded_bound(&limits)), (Some(13), Some(0)));
    assert_eq!((SummaryResult::encoded_bound(&limits), SummaryResult::decoded_bound(&limits)), (Some(273), Some(256)));
    assert_eq!((SummaryEndings::encoded_bound(&limits), SummaryEndings::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((TaskSummary::encoded_bound(&limits), TaskSummary::decoded_bound(&limits)), (Some(1_009), Some(768)));
    assert_eq!((RetentionStage::encoded_bound(&limits), RetentionStage::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((RetentionRow::encoded_bound(&limits), RetentionRow::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((Value::encoded_bound(&limits), Value::decoded_bound(&limits)), (Some(420_721_522), Some(420_734_672)));
    assert_eq!((ModelName::encoded_bound(&limits), ModelName::decoded_bound(&limits)), (Some(4_100), Some(4_096)));
    assert_eq!((Row::encoded_bound(&limits), Row::decoded_bound(&limits)), (Some(420_983_928), Some(420_997_840)));
    assert_eq!((CapacityActionBatch::encoded_bound(&limits), CapacityActionBatch::decoded_bound(&limits)), (Some(13), Some(0)));
    assert_eq!((CapacityActionProposal::encoded_bound(&limits), CapacityActionProposal::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((CapacityActionPersonProposal::encoded_bound(&limits), CapacityActionPersonProposal::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((CapacityActionMessage::encoded_bound(&limits), CapacityActionMessage::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((CapacityActionSource::encoded_bound(&limits), CapacityActionSource::decoded_bound(&limits)), (Some(21), Some(0)));
    assert_eq!((CapacityActionIntroduce::encoded_bound(&limits), CapacityActionIntroduce::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((CapacityActionSubscription::encoded_bound(&limits), CapacityActionSubscription::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((CapacityActionEffect::encoded_bound(&limits), CapacityActionEffect::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((CapacityActionClaim::encoded_bound(&limits), CapacityActionClaim::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((CapacityActionPlace::encoded_bound(&limits), CapacityActionPlace::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((CapacityAction::encoded_bound(&limits), CapacityAction::decoded_bound(&limits)), (Some(22), Some(0)));
    assert_eq!((CapacityCauseTasks::encoded_bound(&limits), CapacityCauseTasks::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseCreationFlights::encoded_bound(&limits), CapacityCauseCreationFlights::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseResourceReports::encoded_bound(&limits), CapacityCauseResourceReports::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseProjectTasks::encoded_bound(&limits), CapacityCauseProjectTasks::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseTree::encoded_bound(&limits), CapacityCauseTree::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseDelegates::encoded_bound(&limits), CapacityCauseDelegates::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseInbox::encoded_bound(&limits), CapacityCauseInbox::decoded_bound(&limits)), (Some(16), Some(0)));
    assert_eq!((CapacityCauseHoldings::encoded_bound(&limits), CapacityCauseHoldings::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseHoldWaiters::encoded_bound(&limits), CapacityCauseHoldWaiters::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseProposals::encoded_bound(&limits), CapacityCauseProposals::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseReferences::encoded_bound(&limits), CapacityCauseReferences::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseSubscriptions::encoded_bound(&limits), CapacityCauseSubscriptions::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseFunders::encoded_bound(&limits), CapacityCauseFunders::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseWriters::encoded_bound(&limits), CapacityCauseWriters::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCausePlacedHolds::encoded_bound(&limits), CapacityCausePlacedHolds::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseProjectPlacedHolds::encoded_bound(&limits), CapacityCauseProjectPlacedHolds::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCausePartyPlacedHolds::encoded_bound(&limits), CapacityCausePartyPlacedHolds::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCauseExemptions::encoded_bound(&limits), CapacityCauseExemptions::decoded_bound(&limits)), (Some(4), Some(0)));
    assert_eq!((CapacityCause::encoded_bound(&limits), CapacityCause::decoded_bound(&limits)), (Some(17), Some(0)));
    assert_eq!((Capacity::encoded_bound(&limits), Capacity::decoded_bound(&limits)), (Some(53), Some(0)));
    assert_eq!((RefusalPersonRevision::encoded_bound(&limits), RefusalPersonRevision::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((LlmFault::encoded_bound(&limits), LlmFault::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((RefusalLlm::encoded_bound(&limits), RefusalLlm::decoded_bound(&limits)), (Some(2), Some(0)));
    assert_eq!((AccountWhy::encoded_bound(&limits), AccountWhy::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((RefusalAccount::encoded_bound(&limits), RefusalAccount::decoded_bound(&limits)), (Some(5), Some(0)));
    assert_eq!((RefusalCircle::encoded_bound(&limits), RefusalCircle::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((RefusalDependencyEnded::encoded_bound(&limits), RefusalDependencyEnded::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((RefusalPlaced::encoded_bound(&limits), RefusalPlaced::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((RefusalHoldOwner::encoded_bound(&limits), RefusalHoldOwner::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((RefusalHoldRevision::encoded_bound(&limits), RefusalHoldRevision::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((TalkOpen::encoded_bound(&limits), TalkOpen::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((RefusalConcluded::encoded_bound(&limits), RefusalConcluded::decoded_bound(&limits)), (Some(8), Some(0)));
    assert_eq!((Refusal::encoded_bound(&limits), Refusal::decoded_bound(&limits)), (Some(9), Some(0)));
    assert_eq!((TaskProblem::encoded_bound(&limits), TaskProblem::decoded_bound(&limits)), (Some(65_615), Some(65_536)));
    assert_eq!((ResultFollowupsDelegates::encoded_bound(&limits), ResultFollowupsDelegates::decoded_bound(&limits)), (Some(67_108_868), Some(67_108_864)));
    assert_eq!((ResultFollowupsProposal::encoded_bound(&limits), ResultFollowupsProposal::decoded_bound(&limits)), (Some(68_157_484), Some(68_157_440)));
    assert_eq!((ResultFollowups::encoded_bound(&limits), ResultFollowups::decoded_bound(&limits)), (Some(68_157_485), Some(68_157_672)));
    assert_eq!((EndFinished::encoded_bound(&limits), EndFinished::decoded_bound(&limits)), (Some(1_572_878), Some(1_572_864)));
    assert_eq!((EndFinishedWithFollowups::encoded_bound(&limits), EndFinishedWithFollowups::decoded_bound(&limits)), (Some(69_730_362), Some(69_730_536)));
    assert_eq!((EndFailed::encoded_bound(&limits), EndFailed::decoded_bound(&limits)), (Some(1), Some(0)));
    assert_eq!((EndFailedWithReason::encoded_bound(&limits), EndFailedWithReason::decoded_bound(&limits)), (Some(133), Some(128)));
    assert_eq!((EndRefusedWithReason::encoded_bound(&limits), EndRefusedWithReason::decoded_bound(&limits)), (Some(132), Some(128)));
    assert_eq!((End::encoded_bound(&limits), End::decoded_bound(&limits)), (Some(69_730_363), Some(69_730_608)));
    assert_eq!((EndRecord::encoded_bound(&limits), EndRecord::decoded_bound(&limits)), (Some(69_730_365), Some(69_730_608)));
    assert_eq!((PhaseRecord::encoded_bound(&limits), PhaseRecord::decoded_bound(&limits)), (Some(2_162_755), Some(2_162_688)));
    assert_eq!((ChangeRecord::encoded_bound(&limits), ChangeRecord::decoded_bound(&limits)), (Some(3), Some(0)));
    assert_eq!((encoded_bound(&limits), decoded_bound(&limits)), (Some(420_983_928), Some(420_997_840)));
}

#[test]
#[expect(clippy::too_many_lines, reason = "Every named limit is reduced from its ceiling, then every root is pinned.")]
#[rustfmt::skip] // Keep each root's two calls and literal bounds on one line.
fn tasks_roots_keep_their_small_limit_bounds() {
    let limits = Limits {
        bytes_parameter_value: CEILINGS.bytes_parameter_value.min(3),
        job_spec_bytes: CEILINGS.job_spec_bytes.min(3),
        job_result_bytes: CEILINGS.job_result_bytes.min(3),
        spec_words: CEILINGS.spec_words.min(3),
        spec_parameters: CEILINGS.spec_parameters.min(3),
        spec_inputs: CEILINGS.spec_inputs.min(3),
        part_segment_bytes: CEILINGS.part_segment_bytes.min(3),
        part_path_segments: CEILINGS.part_path_segments.min(3),
        field_rule_name: CEILINGS.field_rule_name.min(3),
        verdict_fields: CEILINGS.verdict_fields.min(3),
        approval_state: CEILINGS.approval_state.min(3),
        choices_choices: CEILINGS.choices_choices.min(3),
        change_contract_choices: CEILINGS.change_contract_choices.min(3),
        llm_later_subagents: CEILINGS.llm_later_subagents.min(3),
        llm_choice_subagents: CEILINGS.llm_choice_subagents.min(3),
        authority_segment_bytes: CEILINGS.authority_segment_bytes.min(3),
        pattern_segments: CEILINGS.pattern_segments.min(3),
        delegation_kinds: CEILINGS.delegation_kinds.min(3),
        authority_grants: CEILINGS.authority_grants.min(3),
        authority_note_resources: CEILINGS.authority_note_resources.min(3),
        segment_bytes: CEILINGS.segment_bytes.min(3),
        name_path: CEILINGS.name_path.min(3),
        member_dependencies: CEILINGS.member_dependencies.min(3),
        member_holdings: CEILINGS.member_holdings.min(3),
        recurring_template_batch: CEILINGS.recurring_template_batch.min(3),
        result_field_name: CEILINGS.result_field_name.min(3),
        result_field_value: CEILINGS.result_field_value.min(3),
        approved_state: CEILINGS.approved_state.min(3),
        approved_words_some: CEILINGS.approved_words_some.min(3),
        report_result_words: CEILINGS.report_result_words.min(3),
        verdict_result_fields: CEILINGS.verdict_result_fields.min(3),
        verdict_result_words: CEILINGS.verdict_result_words.min(3),
        change_result_title: CEILINGS.change_result_title.min(3),
        change_result_words: CEILINGS.change_result_words.min(3),
        failure_result_reason: CEILINGS.failure_result_reason.min(3),
        cancelled_reason: CEILINGS.cancelled_reason.min(3),
        circle_tasks: CEILINGS.circle_tasks.min(3),
        hold_account_words: CEILINGS.hold_account_words.min(3),
        hold_preparation_words: CEILINGS.hold_preparation_words.min(3),
        rejected_proposal_reason: CEILINGS.rejected_proposal_reason.min(3),
        amendment_instructions_some: CEILINGS.amendment_instructions_some.min(3),
        amendment_dependencies_some: CEILINGS.amendment_dependencies_some.min(3),
        amendment_reason: CEILINGS.amendment_reason.min(3),
        proposed_batch_members: CEILINGS.proposed_batch_members.min(3),
        proposed_settings_items: CEILINGS.proposed_settings_items.min(3),
        proposed_settings_projects: CEILINGS.proposed_settings_projects.min(3),
        proposal_reason: CEILINGS.proposal_reason.min(3),
        settings_instructions_some: CEILINGS.settings_instructions_some.min(3),
        settings_dependencies: CEILINGS.settings_dependencies.min(3),
        history_reason: CEILINGS.history_reason.min(3),
        rejected_person_proposal_reason: CEILINGS.rejected_person_proposal_reason.min(3),
        person_proposal_reason: CEILINGS.person_proposal_reason.min(3),
        grace_reason: CEILINGS.grace_reason.min(3),
        failed_retry_reason_some: CEILINGS.failed_retry_reason_some.min(3),
        refused_retry_reason_some: CEILINGS.refused_retry_reason_some.min(3),
        approval_message_state: CEILINGS.approval_message_state.min(3),
        verdict_message_fields: CEILINGS.verdict_message_fields.min(3),
        word_words: CEILINGS.word_words.min(3),
        activity_words_words: CEILINGS.activity_words_words.min(3),
        kept_line_pending: CEILINGS.kept_line_pending.min(3),
        now_written_line_line: CEILINGS.now_written_line_line.min(3),
        stood_line_words: CEILINGS.stood_line_words.min(3),
        rejected_escalation_reason: CEILINGS.rejected_escalation_reason.min(3),
        recurring_state_last_batch: CEILINGS.recurring_state_last_batch.min(3),
        saved_resource_path: CEILINGS.saved_resource_path.min(3),
        task_history_question_lines: CEILINGS.task_history_question_lines.min(3),
        task_history_proposal_rows: CEILINGS.task_history_proposal_rows.min(3),
        task_history_instructions_some: CEILINGS.task_history_instructions_some.min(3),
        task_history_refused: CEILINGS.task_history_refused.min(3),
        task_run_replies: CEILINGS.task_run_replies.min(3),
        task_definition_dependencies: CEILINGS.task_definition_dependencies.min(3),
        task_definition_waiting_on: CEILINGS.task_definition_waiting_on.min(3),
        task_definition_added: CEILINGS.task_definition_added.min(3),
        task_interests_holdings: CEILINGS.task_interests_holdings.min(3),
        task_interests_delegates: CEILINGS.task_interests_delegates.min(3),
        task_interests_result_followups: CEILINGS.task_interests_result_followups.min(3),
        task_interests_references: CEILINGS.task_interests_references.min(3),
        task_interests_subscriptions: CEILINGS.task_interests_subscriptions.min(3),
        task_interests_inbox: CEILINGS.task_interests_inbox.min(3),
        task_interests_saved: CEILINGS.task_interests_saved.min(3),
        before_talk_started_from: CEILINGS.before_talk_started_from.min(3),
        hot_history_question_lines: CEILINGS.hot_history_question_lines.min(3),
        hot_history_proposal_rows: CEILINGS.hot_history_proposal_rows.min(3),
        hot_history_refused: CEILINGS.hot_history_refused.min(3),
        hot_definition_dependencies: CEILINGS.hot_definition_dependencies.min(3),
        hot_definition_waiting_on: CEILINGS.hot_definition_waiting_on.min(3),
        hot_definition_added: CEILINGS.hot_definition_added.min(3),
        live_hot_started_from: CEILINGS.live_hot_started_from.min(3),
        live_cold_instructions_some: CEILINGS.live_cold_instructions_some.min(3),
        placed_hold_reason: CEILINGS.placed_hold_reason.min(3),
        placed_hold_exempt: CEILINGS.placed_hold_exempt.min(3),
        summary_result_words: CEILINGS.summary_result_words.min(3),
        task_summary_title: CEILINGS.task_summary_title.min(3),
        task_summary_reason: CEILINGS.task_summary_reason.min(3),
        model_name_value: CEILINGS.model_name_value.min(3),
        row_models: CEILINGS.row_models.min(3),
        task_problem_blocked_by_some: CEILINGS.task_problem_blocked_by_some.min(3),
        result_followups_delegates_value: CEILINGS.result_followups_delegates_value.min(3),
        end_failed_with_reason_reason: CEILINGS.end_failed_with_reason_reason.min(3),
        end_refused_with_reason_reason: CEILINGS.end_refused_with_reason_reason.min(3),
    };
    // Roots are the declarations no other declaration names as a field or payload.
    assert_eq!((Row::encoded_bound(&limits), Row::decoded_bound(&limits)), (Some(2_503), Some(2_169)));
    assert_eq!((TaskProblem::encoded_bound(&limits), TaskProblem::decoded_bound(&limits)), (Some(103), Some(24)));
    assert_eq!((EndRecord::encoded_bound(&limits), EndRecord::decoded_bound(&limits)), (Some(391), Some(947)));
    assert_eq!((PhaseRecord::encoded_bound(&limits), PhaseRecord::decoded_bound(&limits)), (Some(107), Some(33)));
    assert_eq!((ChangeRecord::encoded_bound(&limits), ChangeRecord::decoded_bound(&limits)), (Some(3), Some(0)));
    assert_eq!((encoded_bound(&limits), decoded_bound(&limits)), (Some(2_503), Some(2_169)));
}
