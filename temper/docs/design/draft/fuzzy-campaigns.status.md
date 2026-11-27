# fuzzy-campaigns: status

2026-11-27. Where the design lags or departs from [fuzzy-campaigns.md](fuzzy-campaigns.md).

- **Design lags:** 5.1, 7.3 and 7.4: `tools/tiers/campaign.md` takes no `--last`, replays no finding's first case at the last campaign's commit, and a finding's body says nothing of what that replay saw.
- **Design lags:** 4.4 and 6: `temper/docs/design/testing/workloads.md`, 8 gives each sweep one list of kinds and the fewest seeds that make them fall, with no campaign configuration. Its composed workload's draws aren't valid at every seed: a configured kind with no eligible goal panics, and a seed can hold a train member without releasing it. `skein/docs/design/testing/sweeps.md`, 4.2 gives a campaign as many cases as a sweep's gate seeds, with no way to ask for more.
- **Design departs:** 10: the campaign's runner is designed in a document of its own, `tools/tiers/campaign.md`, which `tools/tiers/design.md`, 1 and 10.1 name, where section 10 puts it in `tools/tiers/design.md`: that document is past 64 KB, and a campaign is a concept of its own (`DESIGN.md`, 5). Its designer's choice, 2026-11-27.
- **Design departs:** 9: skein's TLS world, `skein-tls-world::suite`, one of skein's machine worlds, takes no case and stays off `campaign-skein` (`skein/docs/design/testing/README.md`, 7.1): its cases don't replay, since rustls draws from the kernel (`skein/docs/design/net/tls.md`, 5), and a campaign lists only cases that replay (5.2). Its designer's choice, 2026-11-27.
- **Design departs:** 4.4: an adopting sweep whose loop counted no `seed` checks gains one a case, since `cases` counts each (`skein/docs/design/testing/sweeps.md`, 4.2), where 4.4 has its counts stay as they were; no count falls. Its designer's choice, 2026-11-27.
- **Open:** 5.3 and 5.5: which builder runs the daily campaign, within what envelope and under what wall. These are the user's: hardware and figures. Once a day, on a builder apart from the gate, is the user's word (11-26).
