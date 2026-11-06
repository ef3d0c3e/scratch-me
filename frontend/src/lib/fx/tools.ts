import type { ThemeId, ToolId } from '#lib/types.ts';

/** How each scratch tool scrapes, sheds and sounds */
export interface ToolFeel {
	/** Brush diameter, slow to fast, px */
	brush: [number, number];
	/** Colours of what comes off */
	flakes: string[];
	/** A glow around each flake (the laser burns rather than scrapes) */
	glow: string | null;
	/** Multiplies the scratch noise's pitch, and how narrow its band is */
	pitch: number;
	q: number;
}

export const TOOLS: Record<ToolId, ToolFeel> = {
	coin: {
		brush: [20, 30],
		flakes: ['#c9c4dc', '#a9a3c4', '#e9e6f5'],
		glow: null,
		pitch: 1,
		q: 0.9
	},
	key: {
		brush: [15, 22],
		flakes: ['#dfe6ee', '#b8c3cf', '#ffffff'],
		glow: null,
		pitch: 1.5,
		q: 1.8
	},
	eraser: {
		brush: [28, 40],
		flakes: ['#ff9db8', '#f07aa0', '#ffc2d3'],
		glow: null,
		pitch: 0.55,
		q: 0.6
	},
	laser: {
		brush: [16, 24],
		flakes: ['#ff2b4e', '#ff7a45', '#ffd166'],
		glow: '#ff2b4e',
		pitch: 2.2,
		q: 6
	},
	card: {
		brush: [24, 34],
		flakes: ['#c9c4dc', '#e9e6f5', '#57b8ff'],
		glow: null,
		pitch: 1.2,
		q: 1.2
	},
	paw: {
		brush: [26, 38],
		flakes: ['#e9c27d', '#c08a4a', '#fff4dc'],
		glow: null,
		pitch: 0.8,
		q: 0.7
	},
	wand: {
		brush: [18, 28],
		flakes: ['#ffd66b', '#b28cff', '#fff6e5'],
		glow: '#ffd66b',
		pitch: 1.8,
		q: 3
	},
	shovel: {
		brush: [30, 44],
		flakes: ['#c08a4a', '#8a5a2b', '#e9c27d'],
		glow: null,
		pitch: 0.5,
		q: 0.5
	}
};

/** Themes tint the scratch sound a little too */
export const THEME_PITCH: Record<ThemeId, number> = {
	midnight: 1,
	sunset: 0.85,
	mint: 1.1,
	neon: 1.3,
	royal: 0.9,
	ocean: 0.95,
	candy: 1.2,
	arcade: 1.4,
	forest: 0.8,
	champion: 1,
	spooky: 0.7,
	frost: 1.6
};
