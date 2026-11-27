# campaign: status

2026-11-27. Where the code lags or departs from [campaign.md](campaign.md).

- **Not yet:** 2, 4.2, 5, 6 and 9: with `--last`, the campaign still asks git for the last campaign's commit, replays each finding's first case there in a worktree with its own build, and gives each finding what that replay saw, where it reads the last campaign's `campaign.json` alone and marks each finding by whether that campaign held its signature, naming its commit and an early end. [in flight: change 21769]
