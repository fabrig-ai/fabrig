//! Largest values at the ceilings, or at the largest clamp under the cap.
//! Decoded-bound literals use the generated types' sizes on a 64-bit target.

use super::codec_memory::family;
use skein_codec::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[expect(clippy::too_many_lines, reason = "one dispatch per schema root and one clamp per named limit")]
#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        temper_client_protocol_wire::CLIENT_SCHEMA,
        temper_client_protocol_wire::v1,
        roots [
            Trials,
            Approvals,
            Dismissals,
            Aways,
            Decisions,
            Devices,
            FindingAnswer,
            Requesting,
            Forges,
            SettingAnswer,
            ProposalItemsAnswer,
            CharterWords,
            WaysPanel,
            ViewAnswer,
            HistoryAnswer,
            GoalPart,
            ChatPart,
            MilestonePart,
            MilestoneAnswer,
            LimitsAnswer,
            LimitHoldsAnswer,
            Reading,
            PlaygroundAnswer,
            DeploymentAnswer,
            ProgramEnvironmentAnswer,
            LandingPanel,
            LandingsPanel,
            ChangesPanel,
            ProjectStatus,
            Watching,
            ChangePanel,
            Answer,
            LandingRule,
            ForgeCharter,
            TemplateGoal,
            ProjectQuestions,
            ProjectLanding,
            SettingsGate,
            SettingsTier,
            RepositoryGates,
            ProjectGates,
            ProjectWays,
            ProjectClasses,
            DraftsPanel,
            Dependency,
            FollowsPanel,
            ProjectAreas,
            ProjectTriager,
            Welcome,
            Findings,
            AgentFigures,
            LimitOverrides,
        ]
    );
    family!(
        CAP,
        temper_client_protocol_wire::PUBLIC_SCHEMA,
        temper_client_protocol_wire::public_v1,
        roots [
            PublicChange,
            PublicLandings,
            PublicLanding,
        ]
    );
    family!(
        CAP,
        temper_protocol_records::SCHEMA,
        temper_protocol_records::v1,
        roots [
            Stored,
            Pasted,
        ]
    );
    family!(
        CAP,
        temper_protocol_store::OWNER_SET.families.first().expect("store family").current,
        temper_protocol_store::v25,
        roots [
            AnswerName,
            DirectoryEntry,
            Comment,
            Review,
            Status,
            Remark,
            File,
            LandingFollow,
            SettingsGate,
            SettingsTier,
            RepositoryGates,
            Row,
        ]
    );
    family!(
        CAP,
        temper_protocol_worker_wire::SCHEMA,
        temper_protocol_worker_wire::v1,
        roots [
            Items,
            JobSpec,
            Left,
            Saved,
            Preparation,
            Identity,
            Ask,
            Answer,
        ]
    );
}
