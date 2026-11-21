from pathlib import Path
import re
schema=Path('jig/crates/jig-protocol-store/schema/tasks-v13.schema').read_text()
decls=re.findall(r'(?:versioned )?(record|enum) (\w+)\s*\{',schema)
m={}
def lines(s):
 for l in re.split(r'\n(?=(?:record|enum) )', '\n'.join(l.strip() for l in s.strip().splitlines())):
  n=l.split()[1];m[n]=l
lines('''
record Number stands for value
record Person stands for person
record Project
 enum Party = d::Party { task tuple, person tuple }
enum Class = d::Class
record Tries = d::Tries
enum Why by hand
record Role stands for role
 enum PersonAddress = d::PersonAddress { person tuple, role tuple }
enum Executor = d::Executor { person tuple }
record PersonExecutor stands for address
 enum Parameter = d::Parameter
record JobSpec = d::JobSpec
record Spec by hand
enum PartKind = d::PartKind
record PartSegment stands for bytes
record PartPath by hand
enum PartKey = d::PartKey { path tuple, number tuple }
record Part = d::Part
record FieldRule = d::FieldRule
record Verdict by hand
enum Contract by hand
record Tools stands for bits via h::tools
record Models by hand
record ModelEffort = d::ModelEffort
record LlmLater = d::LlmLater { subagents slice }
record LlmChoice = d::LlmChoice { subagents slice }
record Scopes stands for bits via h::scopes
record AuthoritySegment stands for bytes
enum Last = d::Last { exact tuple, open tuple }
record Pattern = d::Pattern { segments slice }
record Grant = d::Grant
record ResourceScope = d::ResourceScope
record Charter stands for charter
record Code stands for code
record AuthorityJob stands for kind
enum AuthorityExecutor = d::AuthorityExecutor { charter tuple, procedure tuple, role tuple, job tuple }
record Delegation = d::Delegation { kinds slice }
record Budget = d::Budget { deadline via s::wall }
record Authority by hand
record Task stands for task
enum Funder = d::Funder { task tuple }
record Numbers = d::Numbers
record PeriodTimes = d::PeriodTimes { began via s::wall, next via s::wall }
record FundingRecord = d::FundingRecord
record Segment stands for bytes
record Name by hand
enum Writer = d::Writer
record WriterSlot = d::WriterSlot
record PoolSlots = d::PoolSlots
enum Holding = d::Holding
enum WakeRule = d::WakeRule
enum ResultsWake = d::ResultsWake
record WakePolicy = d::WakePolicy
enum RecurringOverlap = d::RecurringOverlap
enum Retention = d::Retention
record Closer = d::Closer
enum EndedBy = d::EndedBy { closer tuple }
record Flags = d::Flags
record DeadlineSpan stands for value
enum Deadline = d::Deadline { span tuple }
record Member by hand
record RecurringTemplate by hand
record Creation by hand
record ResultField = d::ResultField
record VerdictResult by hand
record ChangeResult by hand
enum TaskResult by hand
record Done stands for result
enum Ending = d::Ending { done tuple }
record Cancelled { at via s::wall }
record FailedClass stands for class
enum Bound = d::Bound
record Exceeded = d::Exceeded
enum Hold = d::Hold {
    failures tuple
    dependency tuple
    placed { value = hold }
    pool_lost { value = pool }
    exceeded tuple
}
enum Active = d::Active {
    claimed { value = attempt }
    running { value = attempt }
    backing_off { value = until via s::wall }
}
enum Stage = d::Stage { run { value = attempt } }
record Closing = d::Closing
record Activation stands for value
enum Was = d::Was { active tuple, closing tuple }
record Circle = d::Circle { tasks slice }
enum HoldSource = d::HoldSource { resource tuple, circle tuple }
record Held = d::Held { source boxed }
record Final stands for ending
enum Phase = d::Phase { active tuple, closing tuple, held { source boxed }, ended tuple }
enum ProposalKind = d::ProposalKind
enum ProposalHolder = d::ProposalHolder { task tuple, person tuple }
record PendingProposal { since via s::wall }
enum ProposalState = d::ProposalState
record AddressChange = d::AddressChange
enum Tracking = d::Tracking
record Amendment = d::Amendment { dependencies slice }
record ProposedBatch by hand
enum Setting = d::Setting { charter tuple, template tuple, policy tuple, model tuple, worker tuple, provider tuple }
record SettingNumber stands for number
enum SettingEdit = d::SettingEdit
record SettingHead = d::SettingHead
enum ProposalAction by hand
record Proposal = d::Proposal
enum Change = d::Change
record Settings = d::Settings { dependencies slice }
record AmendmentHistory = d::AmendmentHistory
record History = d::History { proposal boxed, at via s::wall, amendment boxed }
record PendingPersonProposal { since via s::wall }
record RejectedPersonProposal { at via s::wall }
record WithdrawnPersonProposal { at via s::wall }
enum PersonProposalState = d::PersonProposalState
enum PersonAction by hand
record PersonProposal = d::PersonProposal
record HoldRow = d::HoldRow { at via s::wall }
record Grace = d::Grace { until via s::wall }
record SettingsRevision = d::SettingsRevision
record Revisions = d::Revisions
enum RetryNotice = d::RetryNotice
enum NoticeState = d::NoticeState
enum NewsClass = d::NewsClass
record VerdictMessage by hand
enum ResultKind by hand
enum SettingsOutcome = d::SettingsOutcome
record ResultMessage stands for kind
enum MessageKind = d::MessageKind { result tuple }
record Word = d::Word { at via s::wall }
enum InvalidResult = d::InvalidResult
enum SuspendWhy = d::SuspendWhy
enum EndKind = d::EndKind
record Suspension = d::Suspension { since via s::wall, until via s::wall }
record AttemptRow = d::AttemptRow { claimed via s::wall, started via s::wall, ended via s::wall }
record OpenAttempt = d::OpenAttempt { claimed via s::wall, started via s::wall }
record HeardMessage stands for word
enum Fresh = d::Fresh
enum SessionEnd = d::SessionEnd
record DecidedProposal = d::DecidedProposal
record KeptLine { pending slice }
enum Line = d::Line { heard tuple }
record ActivityLine = d::ActivityLine { at via s::wall }
enum Status = d::Status
record Stub = d::Stub { result via s::token }
record ReplyHint = d::ReplyHint { at via s::wall }
record EscalationTask stands for task
record EscalationPerson stands for person
enum EscalationHolder = d::EscalationHolder { task tuple, person tuple }
record WaitingEscalation { since via s::wall }
record RejectedEscalation by hand
enum Escalation by hand
record PersonAnswer = d::PersonAnswer { at via s::wall }
record PersonStallNotice = d::PersonStallNotice { at via s::wall }
record PersonWait = d::PersonWait { since via s::wall, deadline via s::wall }
record RecurringState = d::RecurringState { last_batch slice }
record QuestionCredit = d::QuestionCredit
record TimerInterest { at via s::wall }
enum SubscriptionKind = d::SubscriptionKind
record Subscription = d::Subscription
record SavedResource by hand
record AnswerLine = d::AnswerLine
record QuestionLine = d::QuestionLine
record ProposalRow = d::ProposalRow
record Endings = d::Endings
record ReplyObject { message via h::newest, activity via h::newest }
record ProposalUpdate { proposal via h::newest }
enum UpdateObject = d::UpdateObject
record Update = d::Update { at via s::wall }
record ResultRow = d::ResultRow
record Edge = d::Edge
record TranscriptState stands for ever_turned
record TaskHistory by hand
record TaskRun by hand
record Added = d::Added { at via s::wall }
record TaskDefinition by hand
enum InboxItem by hand
record TaskInterests by hand
record RoomProject stands for project
enum LiveRoom = d::LiveRoom { project tuple }
record RoomWait = d::RoomWait
record TaskOutcome by hand
record Talk = d::Talk { opened via s::wall, last via s::wall }
record Session = d::Session { reasons via h::reasons, last via s::wall }
record BeforeTalk by hand
record Concluded = d::Concluded { at via s::wall }
record TaskRecord by hand
record HotHistory by hand
record HotDefinition by hand
record LiveHot by hand
record LiveCold by hand
record EndedByParty = d::EndedByParty { ended_at via s::wall }
record EndedByProject = d::EndedByProject { ended_at via s::wall }
enum HoldOwner = d::Owner { task { value = task } }
enum HoldMode = d::Mode
enum HoldScope = d::Scope
record PlacedHold = d::PlacedHold { exempt slice }
record SummaryCancelled { at via s::wall }
enum SummaryEnding = d::SummaryEnding
enum SummaryResultKind = d::SummaryResultKind
record SummaryResult by hand
record SummaryEndings = d::SummaryEndings
record TaskSummary by hand
enum RetentionStage = d::RetentionStage
record RetentionRow = d::RetentionRow { since via s::wall }
enum Value by hand
record ModelName stands for value
record Row by hand
enum CapacityAction = d::CapacityAction { move_task = Move }
enum CapacityCause = d::CapacityCause
record Capacity = d::Capacity
enum LlmFault = d::LlmFault
enum AccountWhy = d::AccountWhy
record RefusalConcluded { at via s::wall }
enum Refusal = d::Refusal
record TaskProblem = d::Problem { blocked_by slice }
record ResultFollowupsDelegates by hand
record ResultFollowupsProposal stands for value
enum ResultFollowups = d::ResultFollowups { delegates tuple, proposal tuple boxed }
record EndFailed stands for value
record Suspended = d::Suspended
enum End = d::End { exceeded tuple, failed tuple, suspended tuple }
record EndRecord stands for value
record PhaseRecord stands for value
record ChangeRecord stands for value
''')
# One-field payloads retain their inline fields unless a variant uses tuple.
header='''# Tasks translation (skein/docs/design/data/codec/maps.md, 2–5).
records crate::tasks_v13
use d = jig_domain_tasks
use s = skein_codec::convert
use h = crate::codec::tasks
by hand crate::codec::tasks
context crate::codec::tasks::Context

'''
known={n for _,n in decls}
assert not set(m)-known,set(m)-known
Path('jig/crates/jig-protocol-store/schema/tasks.maps').write_text(header+'\n'.join(m.get(n,f'{k} {n}') for k,n in decls)+'\n')
