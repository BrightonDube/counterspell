// cm-chessboard ships no TypeScript declarations. This ambient module gives the
// parts of the API the break room uses a stable, minimal shape.
declare module 'cm-chessboard' {
  export const INPUT_EVENT_TYPE: {
    moveInputStarted: string;
    movingOverSquare: string;
    validateMoveInput: string;
    moveInputFinished: string;
    moveInputCanceled: string;
  };
  export const COLOR: { white: string; black: string };
  export interface MoveInputEvent {
    type: string;
    squareFrom?: string;
    squareTo?: string;
    piece?: { type: string; color: string };
  }
  export class Chessboard {
    constructor(context: HTMLElement, props?: Record<string, unknown>);
    setPosition(fen: string, animated?: boolean): Promise<void>;
    getPosition(): string;
    setPiece(square: string, piece?: string, animated?: boolean): Promise<void>;
    enableMoveInput(
      eventHandler: (event: MoveInputEvent) => boolean | undefined,
      color?: string
    ): void;
    disableMoveInput(): void;
    destroy(): void;
    /** Attached by the Markers extension at mount. */
    addMarker?(
      markerType: { class: string; slice: string },
      square: string
    ): void;
    removeMarkers?(
      markerType?: { class: string; slice: string },
      square?: string
    ): void;
  }
}
