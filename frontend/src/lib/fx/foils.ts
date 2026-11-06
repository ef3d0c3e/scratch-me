import type { FoilId } from '#lib/types.ts';

/** A foil's look: the four stops of its sheen, the ink of its words, a pattern */
export interface FoilLook {
	stops: [string, string, string, string];
	ink: string;
	pattern: 'grain' | 'weave' | 'spots' | 'stars' | 'rainbow';
}

/** Silver is the theme's own foil (its CSS tokens): null here */
export const FOILS: Record<FoilId, FoilLook | null> = {
	silver: null,
	carbon: {
		stops: ['#3a3a42', '#1d1d22', '#55555f', '#141417'],
		ink: 'rgba(255, 255, 255, 0.16)',
		pattern: 'weave'
	},
	leopard: {
		stops: ['#f0c46a', '#d9a441', '#f7d98f', '#c48a2c'],
		ink: 'rgba(60, 30, 0, 0.3)',
		pattern: 'spots'
	},
	gold: {
		stops: ['#ffe08a', '#d4a52b', '#fff3c4', '#b8860b'],
		ink: 'rgba(110, 70, 0, 0.3)',
		pattern: 'grain'
	},
	holo: {
		stops: ['#ffd1f0', '#c3e8ff', '#fff6c2', '#d6c8ff'],
		ink: 'rgba(80, 60, 140, 0.25)',
		pattern: 'rainbow'
	},
	galaxy: {
		stops: ['#1b0b3a', '#0b0420', '#3b1d73', '#05010f'],
		ink: 'rgba(200, 180, 255, 0.22)',
		pattern: 'stars'
	}
};
