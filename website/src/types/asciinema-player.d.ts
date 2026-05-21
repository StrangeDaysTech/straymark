declare module 'asciinema-player' {
  export type AsciinemaPlayerOptions = {
    autoPlay?: boolean;
    loop?: boolean | number;
    preload?: boolean;
    controls?: boolean | 'auto';
    idleTimeLimit?: number;
    fit?: 'width' | 'height' | 'both' | false;
    terminalFontSize?: string;
    cols?: number;
    rows?: number;
    speed?: number;
    poster?: string;
    theme?: string;
    startAt?: number | string;
    pauseOnMarkers?: boolean;
  };

  export type AsciinemaPlayerInstance = {
    play: () => Promise<void>;
    pause: () => void;
    seek: (where: number | string) => void;
    dispose: () => void;
    getCurrentTime: () => number;
    getDuration: () => number | null;
  };

  export function create(
    src: string | object,
    container: HTMLElement,
    options?: AsciinemaPlayerOptions,
  ): AsciinemaPlayerInstance;
}

declare module 'asciinema-player/dist/bundle/asciinema-player.css';
