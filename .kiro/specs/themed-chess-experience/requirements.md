# Themed Chess Experience Requirements

## Goal

Make bot and puzzle chess feel native to Counterspell through responsive scale, Steel Night colours, original lightweight pieces and a living skull opponent.

## Requirements

- Boards use substantially more available space on wide screens and remain fully visible and usable on mobile.
- Board sizing accounts for content width and viewport height.
- Squares, borders, coordinates and move states use semantic Counterspell palette tokens with sufficient contrast.
- Bot and puzzle routes share one themed board configuration.
- Pieces use original lightweight 2.5D vector artwork with outlines, highlights and soft shadows.
- No WebGL, heavy model downloads or real-time 3D engine is allowed.
- A skull opponent appears above the board and has spaced, continuous, compositor-friendly motion while play is active.
- Reduced motion disables continuous movement.
- Legal move input, orientation, bot response, puzzle validation, game status and score persistence remain unchanged.
- The board SVG has an accessible name and move interaction remains keyboard-accessible where supported.
- Artwork loads without delaying move input.
