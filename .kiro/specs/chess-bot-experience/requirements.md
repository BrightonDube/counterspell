# Chess bot experience — requirements

Tracking: #160. Owner alignment completed 2026-11-28.

## R1 — Setup
1. WHEN the chess bot page opens, THE SYSTEM SHALL present six difficulty choices and White/Black controls on the existing page, without a setup dialog or new route.
2. THE SYSTEM SHALL persist the last selected colour and difficulty locally and restore valid preferences; storage failure shall not prevent play.
3. Each level SHALL have genuinely distinct engine settings. Approximate Elo labels SHALL be supported by calibration evidence; uncalibrated estimates must not be presented as established ratings.
4. Starting a new game SHALL apply the chosen settings and invalidate pending work from the previous game.

## R2 — Board and moves
1. THE SYSTEM SHALL preserve square board and piece aspect ratios, current desktop board size where space permits, and avoid horizontal overflow.
2. THE SYSTEM SHALL display complete SAN history, in numbered White/Black columns, with a compact scrollable viewport showing approximately four or five rows.
3. WHEN viewing live play, the list SHALL follow the latest move. Players SHALL be able to inspect past positions and return to the live game without modifying its history or accepting moves in review mode.
4. History SHALL sit beside the board only when sufficient room exists, otherwise below it.
5. Bot and puzzle screens SHALL not expose raw FEN as default visible status. Accessible position, last move and turn information and optional FEN access SHALL remain available.

## R3 — Opponent and outcomes
1. THE SYSTEM SHALL use an original scalable, realistic, scary Halloween SVG skull consistent with Steel Night palette. It SHALL appear close above the board, tilted toward it, with eyes oriented toward the pieces and state-driven subtle reactions instead of repetitive bouncing.
2. THE SYSTEM SHALL communicate turn and thinking state succinctly, including accessible text. No persistent animation SHALL interfere with board interaction.
3. WHEN a player wins, THE SYSTEM SHALL celebrate with confetti and victory feedback once. WHEN a player loses, THE SYSTEM SHALL show a centred large angry red-eyed skull with “You lose. Better luck next time.” Draw feedback SHALL be neutral and show the reason where known.
4. THE SYSTEM SHALL honour reduced motion and provide equivalent static outcome feedback. Result overlays SHALL be dismissible and keyboard accessible.

## R4 — Reliability
Preserve legal chess play, scoring, board orientation, signed-in and signed-out behavior, existing theme and assets outside requested scope. Bound search time on the worker; handle errors and stale responses. Verify regression, responsive browser checks and deployment before declaring completion.
